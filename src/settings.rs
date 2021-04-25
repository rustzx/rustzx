use crate::{
    utils::EmulationSpeed,
    zx::{
        constants::{SCREEN_HEIGHT, SCREEN_WIDTH},
        machine::ZXMachine,
        sound::ay::ZXAYMode,
    },
};
use clap::{App, AppSettings, Arg};
use std::path::{Path, PathBuf};

/// Structure to handle all emulator runtime settings
pub struct RustzxSettings {
    pub machine: ZXMachine,
    pub speed: EmulationSpeed,
    pub fastload: bool,
    pub scale: usize,
    pub screen_size: (usize, usize),
    pub kempston: bool,
    pub ay_mode: ZXAYMode,
    pub ay_enabled: bool,
    pub beeper_enabled: bool,
    pub sound_enabled: bool,
    pub volume: usize,
    pub latency: usize,
    pub rom: Option<PathBuf>,
    pub tap: Option<PathBuf>,
    pub sna: Option<PathBuf>,
}

impl RustzxSettings {
    /// constructs new Settings
    pub fn new() -> RustzxSettings {
        RustzxSettings {
            machine: ZXMachine::Sinclair48K,
            speed: EmulationSpeed::Definite(1),
            fastload: false,
            scale: 2,
            screen_size: (SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2),
            kempston: false,
            ay_mode: ZXAYMode::Mono,
            ay_enabled: false,
            beeper_enabled: true,
            sound_enabled: true,
            volume: 100,
            latency: 1024,
            rom: None,
            tap: None,
            sna: None,
        }
    }

    pub fn from_clap() -> RustzxSettings {
        // get defaults
        let mut out = Self::new();
        // parse cli
        let cmd = App::new("rustzx")
            .setting(AppSettings::ColoredHelp)
            .version(env!("CARGO_PKG_VERSION"))
            .author("Vladislav Nikonov <pacmancoder@gmail.com>")
            .about("ZX Spectrum emulator written in pure Rust")
            // machine settings
            .arg(
                Arg::new("128K")
                    .long("128k")
                    .about("Enables ZX Spectrum 128K mode"),
            )
            .arg(
                Arg::new("FASTLOAD")
                    .short('f')
                    .long("fastload")
                    .about("Accelerates standard tape loaders"),
            )
            // media files
            .arg(
                Arg::new("ROM")
                    .long("rom")
                    .value_name("ROM_PATH")
                    .about("Selects path to rom, otherwise default will be used"),
            )
            .arg(
                Arg::new("TAP")
                    .long("tap")
                    .value_name("TAP_PATH")
                    .about("Selects path to *.tap file"),
            )
            .arg(
                Arg::new("SNA")
                    .long("sna")
                    .value_name("SNA_PATH")
                    .about("Selects path to *.sna snapshot file"),
            )
            // devices
            .arg(Arg::new("KEMPSTON").short('k').long("kempston").about(
                "Enables Kempston joystick. Controlls via arrow keys and \
                 Alt buttons",
            ))
            // emulator settings
            .arg(
                Arg::new("SPEED")
                    .long("speed")
                    .value_name("SPEED_VALUE")
                    .about("Selects speed for emulator in integer multiplier form"),
            )
            .arg(
                Arg::new("SCALE")
                    .long("scale")
                    .value_name("SCALE_VALUE")
                    .about(
                        "Selects default screen size. possible values are positive \
                         integers. Default value is 2",
                    ),
            )
            // sound
            .arg(Arg::new("NOSOUND").long("nosound").about(
                "Disables sound. Use it when you have problems with audio \
                 playback",
            ))
            .arg(
                Arg::new("NOBEEPER")
                    .long("nobeeper")
                    .about("Disables beeper"),
            )
            .arg(
                Arg::new("AY")
                    .long("ay")
                    .value_name("AY_TYPE")
                    .possible_values(&["none", "mono", "abc", "acb"])
                    .about(
                        "Selects AY mode. Use none to disable. \
                         For stereo features use abc or acb, default is mono for \
                         128k and none for 48k.",
                    ),
            )
            .arg(
                Arg::new("VOLUME")
                    .long("volume")
                    .value_name("VOLUME_VALUE")
                    .about(
                        "Selects volume - value in range 0..200. Volume over 100 \
                         can cause sound artifacts",
                    ),
            )
            .arg(
                Arg::new("LATENCY")
                    .long("latency")
                    .short('l')
                    .value_name("SAMPLES")
                    .about(
                        "Selects audio latency. Default is 1024 samples. Set higher \
                         latency if emulator have sound glitches. Or if your \
                         machine can handle this - try to set it lower. Must be \
                         power of two.",
                    ),
            )
            .get_matches();
        // machine type
        if cmd.is_present("128K") {
            out.machine(ZXMachine::Sinclair128K);
        }
        if let Some(speed_str) = cmd.value_of("SPEED") {
            if let Ok(speed) = speed_str.parse::<usize>() {
                out.speed(EmulationSpeed::Definite(speed));
            }
        };
        if let Some(scale_str) = cmd.value_of("SCALE") {
            if let Ok(scale) = scale_str.parse::<usize>() {
                out.scale(scale);
            } else {
                println!("[Warning] Invalid scale factor");
            };
        }
        out.fastload(cmd.is_present("FASTLOAD"))
            .beeper(!cmd.is_present("NOBEEPER"))
            .sound(!cmd.is_present("NOSOUND"))
            .kempston(cmd.is_present("KEMPSTON"));
        if let Some(path) = cmd.value_of_os("ROM") {
            if Path::new(path).is_file() {
                out.rom(path);
            } else {
                println!(
                    "[Warning] ROM file \"{}\" not found",
                    path.to_string_lossy()
                );
            }
        }
        if let Some(path) = cmd.value_of_os("TAP") {
            if Path::new(path).is_file() {
                out.tap(path);
            } else {
                println!(
                    "[Warning] Tape file \"{}\" not found",
                    path.to_string_lossy()
                );
            }
        }
        if let Some(path) = cmd.value_of_os("SNA") {
            if out.machine == ZXMachine::Sinclair48K {
                if Path::new(path).is_file() {
                    out.sna(path);
                } else {
                    println!(
                        "[Warning] Snapshot file \"{}\" not found",
                        path.to_string_lossy()
                    );
                }
            } else {
                println!("[Warning] 128K SNA is not supported!");
            }
        }
        if let Some(value) = cmd.value_of("AY") {
            match value {
                "none" => out.ay(false),
                "mono" => out.ay_mode(ZXAYMode::Mono),
                "abc" => out.ay_mode(ZXAYMode::ABC),
                "acb" => out.ay_mode(ZXAYMode::ACB),
                _ => unreachable!(),
            };
        };
        if let Some(value) = cmd.value_of("VOLUME") {
            if let Ok(value) = value.parse::<usize>() {
                out.volume(value);
            } else {
                println!("[Warning] Volume value is incorrect, setting volume to 100");
            }
        };
        if let Some(latency_str) = cmd.value_of("LATENCY") {
            if let Ok(latency) = latency_str.parse::<usize>() {
                out.latency(latency);
            }
        };
        out
    }

    /// Changes machine type
    pub fn machine(&mut self, machine: ZXMachine) -> &mut Self {
        self.machine = machine;
        match machine {
            ZXMachine::Sinclair48K => self.ay_enabled = false,
            ZXMachine::Sinclair128K => self.ay_enabled = true,
        }
        self
    }

    /// changes screen scale
    pub fn scale(&mut self, scale: usize) -> &mut Self {
        // place into bounds
        if scale > 5 {
            self.scale = 2;
        } else {
            self.scale = scale;
        }
        self.screen_size = (SCREEN_WIDTH * self.scale, SCREEN_HEIGHT * self.scale);
        self
    }

    /// changes fastload flag
    pub fn fastload(&mut self, value: bool) -> &mut Self {
        self.fastload = value;
        self
    }

    /// changes lound latency
    pub fn latency(&mut self, latency: usize) -> &mut Self {
        self.latency = latency;
        self
    }

    /// Changes AY chip mode
    pub fn ay_mode(&mut self, mode: ZXAYMode) -> &mut Self {
        self.ay_enabled = true;
        self.ay_mode = mode;
        self
    }

    /// Changes ay state (on/off)
    pub fn ay(&mut self, state: bool) -> &mut Self {
        self.ay_enabled = state;
        self
    }

    /// Changes beeper state (on/off)
    pub fn beeper(&mut self, state: bool) -> &mut Self {
        self.beeper_enabled = state;
        self
    }

    /// changes sound flag
    pub fn sound(&mut self, state: bool) -> &mut Self {
        self.sound_enabled = state;
        self
    }

    /// Changes volume
    pub fn volume(&mut self, val: usize) -> &mut Self {
        self.volume = if val > 200 { 200 } else { val };
        self
    }

    /// cahnges kempston joy connection
    pub fn kempston(&mut self, value: bool) -> &mut Self {
        self.kempston = value;
        self
    }

    /// changes TAP path
    pub fn tap(&mut self, value: impl AsRef<Path>) -> &mut Self {
        self.tap = Some(value.as_ref().into());
        self
    }

    /// changes SNA path
    pub fn sna(&mut self, value: impl AsRef<Path>) -> &mut Self {
        self.sna = Some(value.as_ref().into());
        self
    }

    /// changes ROM path
    pub fn rom(&mut self, value: impl AsRef<Path>) -> &mut Self {
        self.rom = Some(value.as_ref().into());
        self
    }

    /// changes emulation speed
    pub fn speed(&mut self, value: EmulationSpeed) -> &mut Self {
        self.speed = value;
        self
    }
}
