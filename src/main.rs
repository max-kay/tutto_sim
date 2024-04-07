use tutto_sim::{deck, players::CliPlayer, Game, NaivePlayer};
fn main() {
    let mut game = Game::new(
        vec![
            Box::new(NaivePlayer),
            Box::new(NaivePlayer),
            Box::new(CliPlayer),
        ],
        deck::get_official_cards(),
        None,
    );
    game.play_game();
    game.save_logs()
}
