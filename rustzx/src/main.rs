mod app;
mod host;

use app::{RustzxApp, Settings};
use structopt::StructOpt;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};


fn main() {
    simple_logger::init_with_env().expect("Failed to initialize logger");

    let settings = Settings::from_args();


    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });

    // TODO

    let result = RustzxApp::from_config(settings)
        .and_then(|mut emulator| emulator.start())
        .map_err(|e| {
            log::error!("ERROR: {:#}", e);
        });

    if result.is_err() {
        std::process::exit(1);
    }
}
