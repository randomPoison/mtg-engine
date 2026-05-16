use clap::{Parser, ValueEnum};
use mtg_engine::{
    PlayerAction, PlayerConfig, State, TickEvent,
    card::{CardDefId, CardId, summon_cards_into_existence},
};
use std::{error::Error, fs, num::NonZeroUsize, path::Path};

static GAME_FILE: &str = "game.json";

fn main() -> Result<(), Box<dyn Error>> {
    let command = Command::parse();
    let mut state = load_or_new_state()?;

    if let Some(action) = command.action {
        let player = command.player.expect("required by clap").get() - 1;
        if player >= state.players.len() {
            return Err(format!(
                "invalid player ID {}; current game has {} players",
                player + 1,
                state.players.len()
            )
            .into());
        }

        let action = command.to_player_action(action)?;
        state
            .input(player, action)
            .map_err(|()| Box::<dyn Error>::from("failed to queue player action"))?;
    }

    run_until_input(&mut state);
    save_state(&state)?;

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

fn load_or_new_state() -> Result<State, Box<dyn Error>> {
    if Path::new(GAME_FILE).exists() {
        let game = fs::read_to_string(GAME_FILE)?;
        let mut state: State = serde_json::from_str(&game)?;
        state.card_defs = summon_cards_into_existence();
        Ok(state)
    } else {
        Ok(new_state())
    }
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
    /// One-based player ID.
    #[arg(long, required_unless_present = "action", requires = "action")]
    player: Option<NonZeroUsize>,

    #[arg(long, requires = "player")]
    action: Option<Action>,

    card_id: Option<u32>,
}

#[derive(Clone, Copy, ValueEnum)]
#[value(rename_all = "PascalCase")]
enum Action {
    Pass,
    PlayLand,
}

impl Command {
    fn to_player_action(&self, action: Action) -> Result<PlayerAction, Box<dyn Error>> {
        match action {
            Action::Pass => {
                if let Some(card_id) = self.card_id {
                    Err(format!("Pass does not take a card ID; got {card_id}").into())
                } else {
                    Ok(PlayerAction::PassPriority)
                }
            }
            Action::PlayLand => {
                let card_id = self
                    .card_id
                    .ok_or("PlayLand requires a card ID, e.g. `--action PlayLand 4`")?;
                Ok(PlayerAction::PlayLand(CardId(card_id)))
            }
        }
    }
}
