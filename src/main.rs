use mtg_engine::{Phase, PlayerAction, PlayerConfig, State, TickEvent, card::CardDefId};

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
        println!("turn: {}, event: {event:?}", state.current_player);
        match event {
            TickEvent::Priority(pid) => {
                if pid == state.current_player && state.current_phase == Phase::PreCombat {
                    // Have the current player play one of their cards, then pass priority.
                    let card_id = state.players[state.current_player].hand[0].id();
                    state.input(pid, PlayerAction::PlayLand(card_id)).unwrap();
                    state.input(pid, PlayerAction::PassPriority).unwrap();
                } else {
                    state.input(pid, PlayerAction::PassPriority).unwrap();
                }
            }

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
            TickEvent::PlayCard(_, _card_id) => {}
        }
    }
}
