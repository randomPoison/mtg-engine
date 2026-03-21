pub struct State {
    pub players: Vec<Player>,

    /// The current player, i.e. whose turn it is.
    pub player: PlayerId,

    /// Phase of the current turn.
    pub phase: Phase,

    pub stack: Vec<()>,

    pub priority: Option<Priority>,
}

impl State {
    pub fn tick(&mut self) -> Vec<Event> {
        if let Some(Priority { next, last_actor }) = &mut self.priority {
            let active = *next;

            // If we've made our way back around to the last player to act (or the first
            // player to get priority), then all players have passed priority and we are
            // done with priority.
            if Some(active) == *last_actor {
                self.priority = None;
                return vec![Event::EndPriority];
            }

            // Update the next player, and set last actor if this is the first player to
            // get priority.
            *next = (active + 1) % self.players.len();
            last_actor.get_or_insert(active);

            return vec![Event::Priority(active)];
        }

        match &mut self.phase {
            Phase::Begin(step) => match step {
                BeginStep::Untap => {
                    // CR 501.1: Upkeep comes after untap.
                    *step = BeginStep::Upkeep;

                    // CR: 503.1: At beginning of upkeep, active player gets priority.
                    self.priority = Some(Priority {
                        next: self.player,
                        last_actor: None,
                    });

                    return vec![
                        // CR 502.1: Phased-in permanents phase out, and phased-out permanents phase in.
                        Event::Phase,
                        // CR 502.2: Trigger day/night change.
                        Event::DayNight,
                        // CR 502.3: The active player determines which permanents untap.
                        Event::SelectUntap,
                    ];
                }
                BeginStep::Upkeep => {
                    *step = BeginStep::Draw;
                }
                BeginStep::Draw => {
                    self.phase = Phase::PreCombat;
                }
            },
            Phase::PreCombat => {
                self.phase = Phase::Combat;
            }
            Phase::Combat => {
                self.phase = Phase::PostCombat;
            }
            Phase::PostCombat => {
                self.phase = Phase::End;
            }
            Phase::End => {
                self.player = (self.player + 1) % self.players.len();
                self.phase = Phase::Begin(BeginStep::Untap);
            }
        }

        vec![]
    }
}

type PlayerId = usize;

#[derive(Default)]
pub struct Player {
    pub library: Vec<Card>,
    pub hand: Vec<Card>,
    pub battlefield: Vec<Card>,
    pub graveyard: Vec<Card>,
    pub exile: (),
    pub command: Vec<Card>,
}

pub enum Phase {
    Begin(BeginStep),
    PreCombat,
    Combat,
    PostCombat,
    End,
}

pub enum BeginStep {
    Untap,
    Upkeep,
    Draw,
}

#[derive(Debug)]
pub enum Event {
    // General
    Priority(PlayerId),
    EndPriority,

    // Untap
    Phase,
    DayNight,
    SelectUntap,
}

pub struct Priority {
    pub next: PlayerId,
    pub last_actor: Option<PlayerId>,
}

pub struct Card {
    pub name: String,
    pub cost: Vec<ManaCost>,
    pub kind: CardKind,
}

pub enum CardKind {
    Land,
    Creature,
    Sorcery,
    Artifact,
    Enchantment,
}

pub enum ManaCost {
    /// No mana cost, generally means the card can't be played normally.
    ///
    /// CR: 202.1b Some objects have no mana cost, e.g. lands.
    Nonexistent,

    /// A fixed number of mana of a specified color.
    Fixed { color: ColorCost, quantity: u8 },

    /// A variable amount of mana based on a query of the game state.
    Query,

    /// A variable amount of mana based on some user choice.
    Input,
}

/// CR 202.2a The five colors.
/// CR 202.2b Colorless mana.
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

pub enum ColorCost {
    Single(Color),
    Hybrid(Color, Color),
    Phyrexian(Color),
    Snow,
}
