pub struct State {
    pub players: Vec<Player>,

    /// The current player, i.e. whose turn it is.
    pub player: PlayerId,

    pub stack: Vec<Frame>,
}

impl State {
    pub fn tick(&mut self) -> TickEvent {
        match self.stack.pop().expect("Stack is not empty") {
            Frame::Phase(phase) => match phase {
                Phase::Begin(step) => match step {
                    BeginStep::Untap(event) => match event {
                        UntapEvent::Phasing => {
                            // TODO: Trigger phasing.

                            // CR 502.2: Day/night happens after phasing.
                            self.stack.push(Frame::Phase(Phase::Begin(BeginStep::Untap(
                                UntapEvent::DayNight,
                            ))));

                            return TickEvent::Phase;
                        }
                        UntapEvent::DayNight => {
                            // TODO: Trigger day/night.

                            // CR 502.3: Untap comes after day/night.
                            self.stack.push(Frame::Phase(Phase::Begin(BeginStep::Untap(
                                UntapEvent::SelectUntap,
                            ))));

                            return TickEvent::DayNight;
                        }
                        UntapEvent::SelectUntap => {
                            // TODO: Determine if the current player needs to choose which
                            // permanents to untap, and prompt them to do so if necessary.

                            self.stack.push(Frame::Phase(Phase::Begin(BeginStep::Untap(
                                UntapEvent::Untap,
                            ))));

                            return TickEvent::SelectUntap;
                        }
                        UntapEvent::Untap => {
                            // TODO: Untap all selected permanents.

                            self.stack
                                .push(Frame::Phase(Phase::Begin(BeginStep::Upkeep)));

                            return TickEvent::Untap(vec![]);
                        }
                    },

                    BeginStep::Upkeep => {
                        self.stack.push(Frame::Phase(Phase::Begin(BeginStep::Draw)));

                        // CR: 503.1: At beginning of upkeep, active player gets priority.
                        self.stack.push(Frame::Priority(Priority {
                            next: self.player,
                            last_actor: None,
                        }));

                        return TickEvent::Priority(self.player);
                    }

                    BeginStep::Draw => {
                        // TODO: Draw a card.

                        self.stack.push(Frame::Phase(Phase::PreCombat));

                        return TickEvent::Draw;
                    }
                },
                Phase::PreCombat => {
                    self.stack.push(Frame::Phase(Phase::Combat));
                }
                Phase::Combat => {
                    self.stack.push(Frame::Phase(Phase::PostCombat));
                }
                Phase::PostCombat => {
                    self.stack.push(Frame::Phase(Phase::End));
                }
                Phase::End => {
                    self.player = (self.player + 1) % self.players.len();
                    self.stack.push(Frame::Phase(Phase::Begin(BeginStep::Untap(UntapEvent::Phasing))));
                }
            },

            Frame::Priority(mut priority) => {
                let active = priority.next;

                // If we've made our way back around to the last player to act (or the first
                // player to get priority), then all players have passed priority and we are
                // done with priority.
                if Some(active) == priority.last_actor {
                    return TickEvent::EndPriority;
                }

                // Update the next player, and set last actor if this is the first player to
                // get priority.
                priority.next = (active + 1) % self.players.len();
                priority.last_actor.get_or_insert(active);

                self.stack.push(Frame::Priority(priority));
                return TickEvent::Priority(active);
            }
        }

        unreachable!("Got to end of `tick` without returning an event");
    }
}

pub enum Frame {
    Phase(Phase),
    Priority(Priority),
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
    Untap(UntapEvent),
    Upkeep,
    Draw,
}

pub enum UntapEvent {
    Phasing,
    DayNight,
    SelectUntap,
    Untap,
}

#[derive(Debug)]
pub enum TickEvent {
    // General
    // -------
    /// A player has gained priority.
    Priority(PlayerId),

    /// All players have passed priority and the priority rotation has ended.
    EndPriority,

    // Untap
    // -----
    Phase,
    DayNight,
    SelectUntap,
    Untap(Vec<PermanentId>),

    Draw,
}

pub struct Priority {
    pub next: PlayerId,
    pub last_actor: Option<PlayerId>,
}

#[derive(Debug)]
pub struct Card {
    pub name: String,
    pub cost: Vec<ManaCost>,
    pub r#type: Vec<CardType>,

    /// Some cards have an additional color indicator, separate from their mana
    /// colors. These are generally cards with no mana cost or only colorless
    /// mana cost.
    pub color_indicator: Vec<Color>,
}

#[derive(Debug)]
pub enum CardType {
    Land,
    Creature { subtypes: Vec<String> },
    Sorcery,
    Artifact,
    Enchantment,
}

#[derive(Debug)]
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
#[derive(Debug)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

#[derive(Debug)]
pub enum ColorCost {
    Single(Color),
    Hybrid(Color, Color),
    Phyrexian(Color),
    Snow,
}

pub type PermanentId = ();
