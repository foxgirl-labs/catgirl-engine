#![feature(start)]

#[macro_use] extern crate log;

mod game;
mod server;
mod client;

// Run as Executable (e.g. Linux)
#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    game::setup_logger();
    debug!("Launched as binary...");

    return game::start(_argc, _argv);
}