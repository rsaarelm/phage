#![crate_name="phage"]
#![feature(path_ext)]

extern crate image;

#[macro_use]
extern crate calx;

extern crate world;
extern crate time;

use calx::backend::{Canvas, Event, CanvasBuilder};

use gamestate::GameState;
use titlestate::TitleState;

pub static SCREEN_W: u32 = 640;
pub static SCREEN_H: u32 = 360;

pub mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;
mod gamestate;
mod titlestate;
mod sprite;
mod msg_queue;
mod console;

pub trait State {
    fn process(&mut self, ctx: &mut Canvas, event: Event) -> Option<Transition>;
}

pub enum Transition {
    Game(Option<u32>),
    Title,
    Exit,
}

pub fn version() -> String {
    let next_release = "0.1.0";
    // Set is_release to true for one commit to make a release.
    let is_release = false;

    if is_release {
        format!("{}", next_release)
    } else {
        format!("{}-alpha", next_release)
    }
}

pub fn main() {
    let mut builder = CanvasBuilder::new()
        .set_size(SCREEN_W, SCREEN_H)
        .set_title("Phage")
        .set_frame_interval(0.030f64);
    tilecache::init(&mut builder);

    let mut state: Box<State> = Box::new(TitleState::new());

    let mut canvas = builder.build();
    loop {
        let event = canvas.next_event();
        match state.process(&mut canvas, event) {
            Some(Transition::Title) => { state = Box::new(TitleState::new()); }
            Some(Transition::Game(seed)) => {
                state = Box::new(GameState::new(seed)); }
            Some(Transition::Exit) => { break; }
            _ => ()
        }
    }
}
