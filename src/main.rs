use tetris::game;

fn main() {
    println!("Tetris - Marathon Mode");
    println!(
        "Playfield: {}x{} (visible: {})",
        game::PLAYFIELD_WIDTH,
        game::PLAYFIELD_HEIGHT,
        game::VISIBLE_HEIGHT
    );
}
