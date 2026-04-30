use mtg_engine::{Player, State, card::Card};

fn main() {
    // constructor fn because i dont want `Card` to be `Clone`
    fn library() -> Vec<Card> {
        // Just 5 mountains ig
        // NOt even tgoing to bother with names
        vec![Card(0), Card(0), Card(0), Card(0), Card(0)]
    }

    let mut state = State::new(vec![
        Player {
            library: library(),
            ..Default::default()
        },
        Player {
            library: library(),
            ..Default::default()
        },
    ]);

    loop {
        let event = state.tick();
        dbg!(event);
    }
}
