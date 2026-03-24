use mtg_engine::{BeginStep, Frame, Phase, State, UntapEvent};

fn main() {
    let mut state = State {
        players: vec![Default::default(), Default::default()],
        player: 0,
        stack: vec![Frame::Phase(Phase::Begin(BeginStep::Untap(
            UntapEvent::Phasing,
        )))],
    };

    loop {
        let event = state.tick();
        dbg!(event);
    }
}
