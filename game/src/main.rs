use pixel_zero::meta::embed_metadata;

use crate::game::Game;

mod game;

embed_metadata!(name: "Game", version: 1);

fn main() {
    Game::new().run();
}
