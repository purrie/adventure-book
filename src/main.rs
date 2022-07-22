use game::Game;

extern crate fltk;
extern crate rand;
extern crate regex;

mod game;
mod adventure;
mod file;
mod window;

fn main() {
    let game = Game::create();
    game.start();
}
