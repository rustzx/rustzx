mod app;
mod backends;
mod host;

use app::{RustzxApp, Settings};
use structopt::StructOpt;

fn main() {
    env_logger::init();

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
