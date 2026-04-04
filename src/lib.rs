use card::Card;

pub mod card;

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
            Phase::PreCombat => state.push(MainPhase),
            Phase::Combat => state.push_sequence::<CombatStep>(),
            Phase::PostCombat => state.push(MainPhase),
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
            BeginStep::Upkeep => state.push(Priority {
                next: state.current_player,
                last_actor: None,
            }),
            BeginStep::Draw => {
                state.push(Priority {
                    next: state.current_player,
                    last_actor: None,
                });
                return Some(TickEvent::Draw);
            }
        }

        None
    }
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

pub struct MainPhase;

impl StackFrame for MainPhase {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        state.push(Priority {
            next: state.current_player,
            last_actor: None,
        });
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CombatStep {
    Begin,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    End,
}

impl Sequence for CombatStep {
    const FIRST: Self = Self::Begin;

    fn next(&self) -> Option<Self> {
        use CombatStep::*;
        match self {
            Begin => Some(DeclareAttackers),
            DeclareAttackers => Some(DeclareBlockers),
            DeclareBlockers => Some(CombatDamage),
            CombatDamage => Some(End),
            End => None,
        }
    }
}

impl StackFrame for CombatStep {
    fn eval(&self, _state: &mut State) -> Option<TickEvent> {
        Some(TickEvent::CombatStep(*self))
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

    CombatStep(CombatStep),
}

pub struct Priority {
    pub next: PlayerId,
    pub last_actor: Option<PlayerId>,
}

impl StackFrame for Priority {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        let active = self.next;

        // If we've made our way back around to the last player to act (or the first
        // player to get priority), then all players have passed priority and we are
        // done with priority.
        if Some(active) == self.last_actor {
            return Some(TickEvent::EndPriority);
        }

        // Update the next player, and set last actor if this is the first player to
        // get priority.
        let next = (active + 1) % state.players.len();
        let last_actor = self.last_actor.clone().or(Some(active));
        state.push(Priority { next, last_actor });

        Some(TickEvent::Priority(active))
    }
}

pub type PermanentId = ();

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
