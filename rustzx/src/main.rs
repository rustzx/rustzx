#![allow(dead_code)]

mod app;
mod backends;

use app::RustzxApp;
use rustzx_core::settings::RustzxSettings;

fn main() {
    let settings = RustzxSettings::from_clap();
    RustzxApp::from_config(settings).start();
}
