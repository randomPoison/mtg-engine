use mtg_engine::{Phase, PlayerConfig, State, TickEvent, card::CardDefId};

fn main() {
    fn library() -> Vec<CardDefId> {
        // Just 5 mountains ig
        // NOt even tgoing to bother with names
        vec![
            CardDefId(0),
            CardDefId(0),
            CardDefId(0),
            CardDefId(0),
            CardDefId(0),
        ]
    }

    let mut state = State::new(vec![
        PlayerConfig {
            library: library(),
            ..Default::default()
        },
        PlayerConfig {
            library: library(),
            ..Default::default()
        },
    ]);

    loop {
        let event = state.tick();
        println!("event: {event:?}");
        match event {
            TickEvent::Priority(_) => if state.current_phase == Phase::PreCombat {},

            TickEvent::EndPriority => {}
            TickEvent::BeginTurn(_) => {}
            TickEvent::EndTurn(_) => {}
            TickEvent::BeginPhase(_phase) => {}
            TickEvent::EndPhase(_phase) => {}
            TickEvent::BeginBeginStep(_begin_step) => {}
            TickEvent::EndBeginStep(_begin_step) => {}
            TickEvent::SelectUntap => {}
            TickEvent::Untap => {}
            TickEvent::Draw(_draw) => {}
            TickEvent::CombatStep(_combat_step) => {}
            TickEvent::EndStep(_end_step) => {}
        }
    }
}
