use crate::card::{
    Card, CardDef, CardDefId, CardGen, CardId, CardType, summon_cards_into_existence,
};

pub mod card;
pub mod cards;

pub struct State {
    /// Global list of card definitions.
    pub card_defs: Vec<CardDef>,

    /// State data for all players.
    pub players: Vec<Player>,

    /// The player whose turn is current in progress.
    pub current_player: PlayerId,

    /// The phase of the current turn.
    pub current_phase: Phase,

    /// The state of the in-game stack (not to be confused with internal stack,
    /// which is different).
    pub game_stack: Vec<()>,

    /// Internal state stack, which is, confusingly, different from the in-game stack.
    pub state_stack: Vec<StackFrame>,

    /// Queue of pending actions, listed in reverse order.
    ///
    /// Last item is the next action to process, so we can `pop` elements to
    /// process them in order.
    pub actions: Vec<PendingAction>,
}

impl State {
    pub fn new(players: Vec<PlayerConfig>) -> Self {
        // Initialize the current player to the last player, that way the first time we
        // tick we wrap around to the first player.
        let current_player = players.len() - 1;

        let card_defs = summon_cards_into_existence();

        // This is a mildly janky way of generating the unique IDs for cards, since we
        // have to sift through the initial hands and libraries for all players to
        // generate the IDs. This also doesn't tie cards to the player that owns them.
        // We could probably just build a map for that info, though.
        let mut card_gen = CardGen::new();
        let mut reify_cards =
            |dids: Vec<_>| dids.into_iter().map(|did| card_gen.next(did)).collect();

        let players = players
            .into_iter()
            .map(|PlayerConfig { library, hand }| {
                let library = reify_cards(library);
                let hand = reify_cards(hand);
                Player {
                    library,
                    hand,
                    ..Default::default()
                }
            })
            .collect();

        Self {
            card_defs,
            players,
            current_player,
            current_phase: Phase::Begin,
            state_stack: vec![],
            actions: vec![],
            game_stack: vec![],
        }
    }

    pub fn tick(&mut self) -> TickEvent {
        // TODO: Ordering? Is the last element the one to do first? What does it
        // mean if there are multiple pending actions? Multiple players acting
        // at the same time? Wouldn't that require a priority shift, which
        // implies a `tick` in between the actions?
        if let Some(PendingAction {
            player: pid,
            action,
        }) = self.actions.pop()
        {
            let player = self.players.get_mut(pid).expect("Invalid player ID");

            fn get_priority(state_stack: &mut [StackFrame]) -> Option<&mut Priority> {
                // TODO: The priority may not be the top frame, right?
                match state_stack.last_mut() {
                    Some(StackFrame::Priority(priority)) => Some(priority),
                    _ => None,
                }
            }

            match action {
                PlayerAction::PlayLand(card_id) => {
                    // Validity checks:
                    //
                    // - ✅ Are the player ID and card ID valid? Would be nice to not have to validity check that here,
                    // - ✅ Does the player actually have the card in their hand?
                    // - ✅ Is it actually a land?
                    // - ✅ Is it the player's main phase?
                    // - ✅ Does the player have priority?
                    // - ✅ Is stack empty?
                    // - Have they already played a land this turn? Only one by default
                    //
                    // Would be nice to not panic on invalid inputs, but w/e

                    let card_index = player
                        .hand
                        .iter()
                        .position(|card| card.id() == card_id)
                        .expect("Invalid card ID");
                    let card = &player.hand[card_index];
                    let def = &self.card_defs[card.def().0];
                    assert!(
                        def.r#type.contains(&CardType::Land),
                        "Trying to play a non-land card {card:?} as a land"
                    );

                    assert!(
                        self.current_player == pid,
                        "Can only play lands during your own turn",
                    );
                    assert!(
                        self.current_phase.is_main(),
                        "Can only play lands during your main phase",
                    );
                    assert_eq!(
                        Some(pid),
                        get_priority(&mut self.state_stack).map(|p| p.active),
                        "Can only play a land when you have priority",
                    );
                    assert!(
                        self.game_stack.is_empty(),
                        "Stack must be empty to play a land",
                    );

                    // TODO: Check if we've already played a land this turn.

                    // All validity checks have passed, time to put the land on the field.
                    let card = player.hand.remove(card_index);
                    player.battlefield.push(card);

                    return TickEvent::PlayCard(pid, card_id);
                }

                PlayerAction::PassPriority => {
                    // Validity checks:
                    //
                    // - ✅ There has to be an active priority to pass priority.
                    // - ✅ The passing player has to actually have priority.
                    let priority = get_priority(&mut self.state_stack)
                        .expect("Tried to pass priority when not priority?");
                    assert_eq!(priority.active, pid, "Must have priority to pass");

                    // OKAY we've confirmed that it's valid to pass priority rk, so let's actually do it.
                    let active = priority.active;
                    let next = (active + 1) % self.players.len();

                    if Some(next) == priority.last_actor {
                        // If we've made our way back around to the last player to act (or the first
                        // player to get priority), then all players have passed priority and we are
                        // done with priority.
                        let frame = self
                            .state_stack
                            .pop()
                            .expect("There's a priority frame on the stack");
                        assert!(
                            matches!(frame, StackFrame::Priority(_)),
                            "Popped non-priority frame",
                        );
                    } else {
                        // Update the next player, and set last actor if this is the first player to
                        // get priority.
                        priority.active = next;
                        priority.last_actor.get_or_insert(active);
                    }
                }
            }
        }

        // Process frames on the stack until we evaluate one that emits an event, or
        // until we empty the stack.
        while let Some(frame) = self.state_stack.pop() {
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

    /// Enqueues a player action to be processed at the next `tick`.
    pub fn input(&mut self, player: PlayerId, action: PlayerAction) -> Result<(), ()> {
        // TODO: Validate the input? Is it easy (or even possible) to validate
        // the input now? Or are we going to have to validy check in `tick`?
        self.actions.insert(0, PendingAction { player, action });
        Ok(())
    }

    fn push(&mut self, frame: impl Into<StackFrame>) {
        self.state_stack.push(frame.into());
    }

    fn push_sequence<T: Sequence + Copy>(&mut self)
    where
        StackFrame: From<SequenceFrame<T>>,
    {
        self.push(SequenceFrame::<T>::begin());
    }
}

#[derive(Debug)]
pub enum StackFrame {
    Phase(SequenceFrame<Phase>),
    BeginStep(SequenceFrame<BeginStep>),
    UntapEvent(SequenceFrame<UntapEvent>),
    CombatStep(SequenceFrame<CombatStep>),
    EndStep(SequenceFrame<EndStep>),
    MainPhase(MainPhase),
    Priority(Priority),
}

impl StackFrame {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        match self {
            StackFrame::Phase(frame) => frame.eval(state),
            StackFrame::BeginStep(frame) => frame.eval(state),
            StackFrame::UntapEvent(frame) => frame.eval(state),
            StackFrame::MainPhase(frame) => frame.eval(state),
            StackFrame::CombatStep(frame) => frame.eval(state),
            StackFrame::EndStep(frame) => frame.eval(state),
            StackFrame::Priority(frame) => frame.eval(state),
        }
    }
}

pub trait Sequence: Sized {
    const FIRST: Self;

    fn next(&self) -> Option<Self>;

    fn eval(&self, state: &mut State) -> Option<TickEvent>;

    fn begin_event(&self) -> Option<TickEvent> {
        None
    }

    fn end_event(&self) -> Option<TickEvent> {
        None
    }
}

#[derive(Debug)]
pub struct SequenceFrame<T> {
    seq: T,
    step: SequenceStep,
}

#[derive(Debug)]
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

impl<T: Sequence + Copy> SequenceFrame<T>
where
    StackFrame: From<SequenceFrame<T>>,
{
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

impl From<SequenceFrame<Phase>> for StackFrame {
    fn from(frame: SequenceFrame<Phase>) -> Self {
        Self::Phase(frame)
    }
}

impl From<SequenceFrame<BeginStep>> for StackFrame {
    fn from(frame: SequenceFrame<BeginStep>) -> Self {
        Self::BeginStep(frame)
    }
}

impl From<SequenceFrame<UntapEvent>> for StackFrame {
    fn from(frame: SequenceFrame<UntapEvent>) -> Self {
        Self::UntapEvent(frame)
    }
}

impl From<SequenceFrame<CombatStep>> for StackFrame {
    fn from(frame: SequenceFrame<CombatStep>) -> Self {
        Self::CombatStep(frame)
    }
}

impl From<SequenceFrame<EndStep>> for StackFrame {
    fn from(frame: SequenceFrame<EndStep>) -> Self {
        Self::EndStep(frame)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Begin,
    PreCombat,
    Combat,
    PostCombat,
    End,
}

impl Phase {
    pub fn is_main(self) -> bool {
        matches!(self, Phase::PreCombat | Phase::PostCombat)
    }
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

    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        state.current_phase = *self;
        match self {
            Phase::Begin => state.push_sequence::<BeginStep>(),
            Phase::PreCombat => state.push(MainPhase),
            Phase::Combat => state.push_sequence::<CombatStep>(),
            Phase::PostCombat => state.push(MainPhase),
            Phase::End => state.push_sequence::<EndStep>(),
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

    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        match self {
            BeginStep::Untap => {
                state.push_sequence::<UntapEvent>();
            }
            BeginStep::Upkeep => state.push(Priority {
                active: state.current_player,
                last_actor: None,
            }),
            BeginStep::Draw => {
                let player = &mut state.players[state.current_player];

                // Draw a card from the current player's library.
                let Some(draw) = player.library.pop() else {
                    todo!("Player ran out of cards to draw, they lose the game");
                };
                let draw_id = draw.id();
                player.hand.push(draw);

                state.push(Priority {
                    active: state.current_player,
                    last_actor: None,
                });

                return Some(TickEvent::Draw(draw_id));
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
pub struct MainPhase;

impl From<MainPhase> for StackFrame {
    fn from(frame: MainPhase) -> Self {
        Self::MainPhase(frame)
    }
}

impl MainPhase {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        state.push(Priority {
            active: state.current_player,
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

    fn eval(&self, _state: &mut State) -> Option<TickEvent> {
        Some(TickEvent::CombatStep(*self))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EndStep {
    End,
    Cleanup,
}

impl Sequence for EndStep {
    const FIRST: Self = Self::End;

    fn next(&self) -> Option<Self> {
        use EndStep::*;
        match self {
            End => Some(Cleanup),
            Cleanup => None,
        }
    }

    fn eval(&self, _state: &mut State) -> Option<TickEvent> {
        Some(TickEvent::EndStep(*self))
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

    Draw(CardId),

    CombatStep(CombatStep),

    EndStep(EndStep),

    /// A card entered the battlefield.
    PlayCard(PlayerId, CardId),
}

#[derive(Debug, Clone)]
pub struct Priority {
    pub active: PlayerId,
    pub last_actor: Option<PlayerId>,
}

impl From<Priority> for StackFrame {
    fn from(frame: Priority) -> Self {
        Self::Priority(frame)
    }
}

impl Priority {
    fn eval(&self, state: &mut State) -> Option<TickEvent> {
        // Nothing happens on tick, we have to wait for a player to pass
        // priority or otherwise perform an action.
        state.push(self.clone());
        Some(TickEvent::Priority(self.active))
    }
}

pub type PermanentId = ();

type PlayerId = usize;

#[derive(Default)]
pub struct Player {
    /// Cards in the player's library, listed from bottom to top.
    pub library: Vec<Card>,
    pub hand: Vec<Card>,
    pub battlefield: Vec<Card>,
    pub graveyard: Vec<Card>,
    pub exile: (),
    pub command: Vec<Card>,
}

#[derive(Default)]
pub struct PlayerConfig {
    pub library: Vec<CardDefId>,
    pub hand: Vec<CardDefId>,
}

#[derive(Debug)]
pub enum PlayerAction {
    PassPriority,
    PlayLand(CardId),
}

pub struct PendingAction {
    pub player: PlayerId,
    pub action: PlayerAction,
}
