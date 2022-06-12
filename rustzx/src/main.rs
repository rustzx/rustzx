mod app;
mod backends;
mod host;

use app::{RustzxApp, Settings};
use structopt::StructOpt;

fn main() {
    simple_logger::init_with_env().expect("Failed to initialize logger");

    let settings = Settings::from_args();
    let result = RustzxApp::from_config(settings)
        .and_then(|mut emulator| emulator.start())
        .map_err(|e| {
            log::error!("ERROR: {:#}", e);
        });

    if result.is_err() {
        std::process::exit(1);
    }
}
