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
        let current_player = players.len() - 1;

        Self {
            players,
            current_player,
            stack: vec![],
        }
    }

    pub fn tick(&mut self) -> TickEvent {
        // Process frames on the stack until we evaluate one that emits an event, or
        // until we empty the stack.
        while let Some(frame) = self.stack.pop() {
            if let Some(event) = frame.eval(self) {
                return event;
            }
        }

        // No more frames on the stack, the current turn is done. Setup for the next
        // turn and notify that the next turn is starting.
        //
        // NOTE: It might be nice to have the turn be just another frame on the stack,
        // but determining the next player in the sequence depends on the game state
        // (i.e. the list of living players and their ordering). Right now we don't
        // provide the state when walking a sequence, so we handle the turn directly.
        //
        // TODO: Account for players that have been killed and so aren't part of the
        // turn rotation. We may be able to handle this by just removing the dead
        // players from the players list, but with the current setup that would mean
        // that the player IDs change as players are knocked out.
        let next = (self.current_player + 1) % self.players.len();

        self.current_player = next;
        self.push_sequence::<Phase>();

        TickEvent::BeginTurn(next)
    }

    fn push(&mut self, frame: impl StackFrame) {
        self.stack.push(Box::new(frame));
    }

    fn push_sequence<T: Sequence + StackFrame + Copy>(&mut self) {
        self.push(SequenceFrame::<T>::begin());
    }
}

pub trait StackFrame: 'static {
    fn eval(&self, state: &mut State) -> Option<TickEvent>;
}

pub trait Sequence: Sized {
    const FIRST: Self;

    fn next(&self) -> Option<Self>;

    fn begin_event(&self) -> Option<TickEvent> {
        None
    }

    fn end_event(&self) -> Option<TickEvent> {
        None
    }
}

pub struct SequenceFrame<T> {
    seq: T,
    step: SequenceStep,
}

pub enum SequenceStep {
    Begin,
    Eval,
    End,
}

impl<T: Sequence> SequenceFrame<T> {
    pub fn begin() -> Self {
        Self {
            seq: T::FIRST,
            step: SequenceStep::Begin,
        }
    }
}

impl<T: Sequence + StackFrame + Copy> StackFrame for SequenceFrame<T> {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        match self.step {
            SequenceStep::Begin => {
                state.push(SequenceFrame {
                    seq: self.seq,
                    step: SequenceStep::Eval,
                });
                self.seq.begin_event()
            }

            SequenceStep::Eval => {
                state.push(SequenceFrame {
                    seq: self.seq,
                    step: SequenceStep::End,
                });
                self.seq.eval(state)
            }

            SequenceStep::End => {
                if let Some(next) = self.seq.next() {
                    state.push(SequenceFrame {
                        seq: next,
                        step: SequenceStep::Begin,
                    });
                }
                self.seq.end_event()
            }
        }
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

    fn begin_event(&self) -> Option<TickEvent> {
        Some(TickEvent::BeginPhase(*self))
    }

    fn end_event(&self) -> Option<TickEvent> {
        Some(TickEvent::EndPhase(*self))
    }
}

impl StackFrame for Phase {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        match self {
            Phase::Begin => {
                state.push_sequence::<BeginStep>();
            }
            Phase::PreCombat => todo!(),
            Phase::Combat => todo!(),
            Phase::PostCombat => todo!(),
            Phase::End => todo!(),
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BeginStep {
    Untap,
    Upkeep,
    Draw,
}

impl Sequence for BeginStep {
    const FIRST: Self = Self::Untap;

    fn next(&self) -> Option<Self> {
        use BeginStep::*;
        match self {
            Untap => Some(Upkeep),
            Upkeep => Some(Draw),
            Draw => None,
        }
    }

    fn begin_event(&self) -> Option<TickEvent> {
        Some(TickEvent::BeginBeginStep(*self))
    }

    fn end_event(&self) -> Option<TickEvent> {
        Some(TickEvent::EndBeginStep(*self))
    }
}

impl StackFrame for BeginStep {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        match self {
            BeginStep::Untap => {
                state.push_sequence::<UntapEvent>();
            }
            BeginStep::Upkeep => todo!(),
            BeginStep::Draw => todo!(),
        }

        None
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

#[derive(Debug, Clone, Copy)]
pub enum UntapEvent {
    Phasing,
    DayNight,
    SelectUntap,
    Untap,
}

impl Sequence for UntapEvent {
    const FIRST: Self = Self::Phasing;

    fn next(&self) -> Option<Self> {
        use UntapEvent::*;
        match self {
            Phasing => Some(DayNight),
            DayNight => Some(SelectUntap),
            SelectUntap => Some(Untap),
            Untap => None,
        }
    }
}

impl StackFrame for UntapEvent {
    fn eval(&self, _state: &mut State) -> Option<TickEvent> {
        match self {
            UntapEvent::Phasing => None,
            UntapEvent::DayNight => None,
            UntapEvent::SelectUntap => Some(TickEvent::SelectUntap),
            UntapEvent::Untap => Some(TickEvent::Untap),
        }
    }
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

    BeginBeginStep(BeginStep),
    EndBeginStep(BeginStep),

    SelectUntap,
    Untap,

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
