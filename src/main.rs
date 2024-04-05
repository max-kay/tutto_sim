use tutto_sim::{deck, Game, NaivePlayer};
fn main() {
    let mut game = Game::new(
        vec![Box::new(NaivePlayer), Box::new(NaivePlayer)],
        deck::get_official_cards(),
        None,
    );
    game.play_game();
    game.save_logs()
}
