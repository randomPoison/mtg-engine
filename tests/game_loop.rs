use mtg_engine::{PlayerAction, PlayerConfig, State, TickEvent, card::CardDefId};

fn mountains(count: usize) -> Vec<CardDefId> {
    vec![CardDefId(0); count]
}

#[test]
fn game_loop_runs_until_player_decks() {
    let mut state = State::new(vec![
        PlayerConfig {
            library: mountains(2),
            ..Default::default()
        },
        PlayerConfig {
            library: mountains(2),
            ..Default::default()
        },
    ]);

    for _ in 0..1_000 {
        let event = state.tick();

        match event {
            TickEvent::Priority(player) => state
                .input(player, PlayerAction::PassPriority)
                .expect("priority pass should be queued"),
            TickEvent::LoseGame(player) => {
                assert_eq!(player, 0);
                assert!(state.players[0].library.is_empty());
                assert!(state.players[1].library.is_empty());
                assert_eq!(state.players[0].hand.len(), 2);
                assert_eq!(state.players[1].hand.len(), 2);
                return;
            }
            _ => {}
        }
    }

    panic!("game did not end within 1000 ticks");
}
