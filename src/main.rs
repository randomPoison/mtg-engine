use mtg_engine::State;

fn main() {
    let mut state = State::new(vec![Default::default(), Default::default()]);

    loop {
        let event = state.tick();
        dbg!(event);
    }
}
