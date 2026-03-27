pub struct State {
    pub players: Vec<Player>,

    /// The player whose turn is current in progress.
    pub current_player: PlayerId,

    pub stack: Vec<Box<dyn StackFrame>>,
}

impl State {
    pub fn new(players: Vec<Player>) -> Self {
        // Initialize the current player to the last player, that way the first time we
        // tick we wrap around the first player.
        let current_player = players.len();

        Self {
            players,
            current_player,
            stack: vec![],
        }
    }

    pub fn tick(&mut self) -> TickEvent {
        // Process the next frame on the stack, or move on to the next turn.
        match self.stack.pop() {
            Some(frame) => frame.eval(self),

            // No more frames on the stack, the current turn is done. Setup for the next
            // turn and notify that the next turn is starting.
            //
            // NOTE: It might be nice to have the turn be just another frame on the stack,
            // but determining the next player in the sequence depends on the game state
            // (i.e. the list of living players and their ordering). Right now we don't
            // provide the state when walking a sequence, so we handle the turn directly.
            None => {
                // TODO: Account for players that have been killed and so aren't part of the
                // turn rotation. We may be able to handle this by just removing the dead
                // players from the players list, but with the current setup that would mean
                // that the player IDs change as players are knocked out.
                let next = (self.current_player + 1) % self.players.len();

                self.current_player = next;
                self.push(SequenceFrame(Phase::Begin));

                TickEvent::BeginTurn(next)
            }
        }
    }

    fn push(&mut self, frame: impl StackFrame) {
        self.stack.push(Box::new(frame));
    }
}

pub trait StackFrame: 'static {
    fn eval(&self, state: &mut State) -> TickEvent;
}

pub trait Sequence: Sized {
    const FIRST: Self;
    fn next(&self) -> Option<Self>;
}

pub struct SequenceFrame<T>(T);

impl<T: Sequence + StackFrame> StackFrame for SequenceFrame<T> {
    fn eval(&self, state: &mut State) -> TickEvent {
        // First push the next step in the sequence onto the stack.
        if let Some(next) = self.0.next() {
            state.push(Self(next));
        }

        // Then allow the current step in the sequence to do whatever it wants.
        self.0.eval(state)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Phase {
    Begin,
    PreCombat,
    Combat,
    PostCombat,
    End,
}

impl Sequence for Phase {
    const FIRST: Self = Self::Begin;

    fn next(&self) -> Option<Phase> {
        use Phase::*;
        match self {
            Begin => Some(PreCombat),
            PreCombat => Some(Combat),
            Combat => Some(PostCombat),
            PostCombat => Some(End),
            End => None,
        }
    }
}

impl StackFrame for Phase {
    fn eval(&self, _state: &mut State) -> TickEvent {
        TickEvent::BeginPhase(*self)
    }
}

pub enum BeginFrame {}

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

pub enum UntapEvent {
    Phasing,
    DayNight,
    SelectUntap,
    Untap,
}

#[derive(Debug)]
pub enum TickEvent {
    /// A player has gained priority.
    Priority(PlayerId),

    /// All players have passed priority and the priority rotation has ended.
    EndPriority,

    BeginTurn(PlayerId),
    EndTurn(PlayerId),

    BeginPhase(Phase),
    EndPhase(Phase),

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
