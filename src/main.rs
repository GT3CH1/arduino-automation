use std::env;

extern crate firebase;

mod daemon;
mod models;
mod consts;

fn main() {
    env_logger::init();
    daemon::run();
}