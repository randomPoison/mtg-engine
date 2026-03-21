pub struct State {
    pub players: Vec<()>,

    /// The current player, i.e. whose turn it is.
    pub player: PlayerId,

    /// Phase of the current turn.
    pub phase: Phase,

    pub stack: Vec<()>,
}

impl State {
    pub fn tick(&mut self) -> Vec<Event> {
        match &mut self.phase {
            Phase::Begin(step) => match step {
                BeginStep::Untap => {
                    *step = BeginStep::Upkeep;
                    return vec![Event::Phase, Event::DayNight, Event::SelectUntap];
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
    // Untap events.
    Phase,
    DayNight,
    SelectUntap,
}
