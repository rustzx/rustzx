use time;
use z80::Z80;
use zx::{ZXController, ZXMachine};
use utils::EmulationSpeed;

pub struct Emulator {
    machine: ZXMachine,
    cpu: Z80,
    pub controller: ZXController,
    speed: EmulationSpeed,
}

impl Emulator {
    pub fn new(machine: ZXMachine) -> Emulator {
        Emulator {
            machine: machine,
            cpu: Z80::new(),
            controller: ZXController::new(machine),
            speed: EmulationSpeed::Definite(1),
        }
    }

    pub fn set_speed(&mut self, new_speed: EmulationSpeed) {
        self.speed = new_speed;
    }

    /// Emulate frames, maximum in `max_time` time, returns emulation time in nanoseconds
    /// in most cases time is max 1/50 of second, even when using
    /// loader acceleration
    pub fn emulate_frame(&mut self, max_time: u64) -> u64 {
        let mut time = 0u64;
        'frame: loop {
            // start of current frame
            let start_time = time::precise_time_ns();
            // reset controller internal frame counter
            self.controller.reset_frame_counter();
            'cpu: loop {
                self.cpu.emulate(&mut self.controller);
                // If speed is defined
                if let EmulationSpeed::Definite(multiplier) = self.speed {
                    if self.controller.frames_count() >= multiplier {
                        // no more frames
                        return time::precise_time_ns() - start_time;
                    };
                // if speed is maximal.
                } else {
                    // if any frame passed then break cpu loop, but try to start new frame
                    if self.controller.frames_count() != 0 {
                        break 'cpu;
                    }
                }
            };
            time += time::precise_time_ns() - start_time;
            // if time is bigger than `max_time` then stop emulation cycle
            if time > max_time {
                break 'frame;
            }
        };
        return time;
    }
}
