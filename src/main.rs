use clap::{Parser, Subcommand, ValueEnum};
use mtg_engine::{
    PlayerAction, PlayerConfig, StackFrame, State, TickEvent,
    card::{CardDefId, CardId, summon_cards_into_existence},
};
use std::{error::Error, fs, num::NonZeroUsize, path::Path};

static GAME_FILE: &str = "game.json";

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let command = Command::parse();

    match command.command {
        SubCommand::Start { force } => {
            if Path::new(GAME_FILE).exists() && !force {
                return Err(format!(
                    "{GAME_FILE} already exists; run `mtg start --force` to overwrite it"
                )
                .into());
            }

            let mut state = new_state();
            run_until_input(&mut state);
            save_state(&state)?;
        }

        SubCommand::Show => {
            let state = load_state()?;
            println!(
                "Player: {}, Phase: {:?}",
                state.current_player + 1,
                state.current_phase
            );

            if let StackFrame::Priority(priority) =
                state.state_stack.last().expect("Stack can't be empty")
            {
                println!("  Player {} has priority", priority.active + 1);
            }

            for (pid, player) in state.players.iter().enumerate() {
                println!("Player {} battlefield:", pid + 1);
                for card in &player.battlefield {
                    let def = &state.card_defs[card.def().0];
                    println!("  {}", def.name);
                }
            }
        }

        SubCommand::Action {
            player,
            action,
            card_id,
        } => {
            let mut state = load_state()?;
            let player = player.get() - 1;
            if player >= state.players.len() {
                return Err(format!(
                    "invalid player ID {}; current game has {} players",
                    player + 1,
                    state.players.len()
                )
                .into());
            }

            let action = to_player_action(action, card_id)?;
            state
                .input(player, action)
                .map_err(|()| Box::<dyn Error>::from("failed to queue player action"))?;

            run_until_input(&mut state);
            save_state(&state)?;
        }
    }

    Ok(())
}

fn run_until_input(state: &mut State) {
    loop {
        let event = state.tick();
        println!(
            "turn: {}, phase: {:?}, event: {event:?}",
            state.current_player + 1,
            state.current_phase,
        );

        if matches!(event, TickEvent::Priority(_)) {
            break;
        }
    }
}

fn load_state() -> Result<State, Box<dyn Error>> {
    if !Path::new(GAME_FILE).exists() {
        return Err(format!("no {GAME_FILE} found; run `mtg start` to create a new game").into());
    }

    let game = fs::read_to_string(GAME_FILE)?;
    let mut state: State = serde_json::from_str(&game)?;
    state.card_defs = summon_cards_into_existence();
    Ok(state)
}

fn save_state(state: &State) -> Result<(), Box<dyn Error>> {
    let game = serde_json::to_string_pretty(state)?;
    fs::write(GAME_FILE, game)?;
    Ok(())
}

fn new_state() -> State {
    fn library() -> Vec<CardDefId> {
        vec![
            CardDefId(0),
            CardDefId(0),
            CardDefId(0),
            CardDefId(0),
            CardDefId(0),
        ]
    }

    State::new(vec![
        PlayerConfig {
            library: library(),
            ..Default::default()
        },
        PlayerConfig {
            library: library(),
            ..Default::default()
        },
    ])
}

#[derive(Parser)]
#[command(name = "mtg")]
struct Command {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    Start {
        /// Overwrite an existing game.json.
        #[arg(long)]
        force: bool,
    },
    Show,
    Action {
        /// One-based player ID.
        player: NonZeroUsize,

        action: Action,

        card_id: Option<u32>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
#[value(rename_all = "PascalCase")]
enum Action {
    Pass,
    PlayLand,
}

fn to_player_action(action: Action, card_id: Option<u32>) -> Result<PlayerAction, Box<dyn Error>> {
    match action {
        Action::Pass => {
            if let Some(card_id) = card_id {
                Err(format!("Pass does not take a card ID; got {card_id}").into())
            } else {
                Ok(PlayerAction::PassPriority)
            }
        }
        Action::PlayLand => {
            let card_id =
                card_id.ok_or("PlayLand requires a card ID, e.g. `mtg action 1 PlayLand 4`")?;
            Ok(PlayerAction::PlayLand(CardId(card_id)))
        }
    }
}
