use mtg_engine::{Phase, State};

fn main() {
    let mut state = State {
        players: vec![(), ()],
        player: 0,
        phase: Phase::Begin(mtg_engine::BeginStep::Untap),
        stack: vec![],
        priority: None,
    };

    loop {
        let events = state.tick();
        for event in events {
            dbg!(event);
        }
    }
}
