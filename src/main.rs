use mtg_engine::Phase;

fn main() {
    let players = [(), ()];

    let mut current = 0;
    let mut phase = Phase::Begin;
    loop {
        match phase {
            Phase::Begin => {
                phase = Phase::PreCombat;
            }
            Phase::PreCombat => {
                phase = Phase::Combat;
            }
            Phase::Combat => {
                phase = Phase::PostCombat;
            }
            Phase::PostCombat => {
                phase = Phase::End;
            }
            Phase::End => {
                current = (current + 1) % players.len();
                phase = Phase::Begin;
            }
        }
    }
}
