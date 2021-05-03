#![allow(dead_code)]

mod app;
mod backends;
mod host;

use app::{ RustzxApp, Settings };

fn main() {
    env_logger::init();

    let settings = Settings::from_clap();
    let result = RustzxApp::from_config(settings)
        .map_err(|e| log::error!("ERROR: {}", e))
        .and_then(|mut emulator| {
            emulator.start();
            Ok(())
        });

    if result.is_err() {
        std::process::exit(1);
    }
}
