use rustzx_core::{
    zx::{machine::ZXMachine, sound::ay::ZXAYMode},
    EmulationMode, RustzxSettings,
};

use std::path::PathBuf;

use structopt::StructOpt;

/// Structure to handle all emulator runtime settings
#[derive(StructOpt)]
pub struct Settings {
    /// Specify machine type for launch. Possible values:
    ///   [`48k`, `48`] - Sinclair ZX Spectrum 48K
    ///   [`128k`, `128`] - Sinclair ZX Spectrum 128K
    #[structopt(verbatim_doc_comment, short, long, default_value = "48k", parse(try_from_str = machine_from_str))]
    pub machine: ZXMachine,
    /// Set emulation speed at emualtor start-up. Can be specified as deciamal non-zero
    /// value or as a special value `MAX` to run emulator as fast as possible
    #[structopt(long, default_value = "1", parse(try_from_str = emualtion_speed_from_str))]
    pub speed: EmulationMode,
    /// Disable fast tape loading
    #[structopt(long = "nofastload")]
    pub disable_fastload: bool,
    /// Disable automatic tape loading via placing emulator to tape load state immediately
    /// after launch
    #[structopt(long = "noautoload")]
    pub disable_autoload: bool,
    /// Set windows scale for emulator. Can be set as decimal non-zero value. Defaults to 2
    #[structopt(short, long, default_value = "2", parse(try_from_str = scale_from_str))]
    pub scale: usize,
    /// Disable kempston joy support. If enabled, arrow and `Alt` keys are bound by default
    /// to the kempston joy
    #[structopt(long = "nokempston")]
    pub disable_kempston: bool,
    /// Enables kempston mouse support. If enabled, locks mouse in application
    #[structopt(long = "mouse")]
    pub enable_mouse: bool,
    /// Sets mouse sensitivity [1..=100]. Defaults to 20
    #[structopt(long = "mouse-sensitivity", default_value = "20")]
    pub mouse_sensitivity: usize,
    /// Set AY-3-8910 sound chip mode. Can be set to `mono`, `abc`(stereo) or `acb`(stereo)
    /// Defaults to `abc`
    #[structopt(long, default_value = "abc", parse(try_from_str = ay_mode_from_str))]
    /// Disable AY-3-8910 chip support
    pub ay_mode: ZXAYMode,
    /// Force enable AY-3-8910 chip on unsupported machines
    #[structopt(long = "ay", conflicts_with = "force-disable-ay")]
    pub force_enable_ay: bool,
    /// Force disable AY-3-8910 chip on supported systems
    #[structopt(long = "noay", conflicts_with = "force-enable-ay")]
    pub force_disable_ay: bool,
    /// Disable beeper
    #[structopt(long = "nobeeper")]
    pub disable_beeper: bool,
    /// Disable sound
    #[structopt(long = "nosound")]
    pub disable_sound: bool,
    /// Set custom sound latency. Defaults to 1024 samples
    #[structopt(long, default_value = "1024", parse(try_from_str = sound_latency_from_str))]
    pub sound_latency: usize,
    /// Set custom sound sample rate. Defaults to 44100 samples per second
    #[structopt(long, default_value = "44100", parse(try_from_str = sound_sample_rate_from_str))]
    pub sound_sample_rate: usize,

    /// Set path to custom rom file. in case of multipart ROMs for 128k, the first part file,
    /// extension of which should end with `.0`
    #[structopt(long, conflicts_with = "file-autodetect")]
    pub rom: Option<PathBuf>,
    /// Set tape file path. Only `.tap` files are supported currently
    #[structopt(long, conflicts_with = "file-autodetect")]
    pub tape: Option<PathBuf>,
    /// Set snapshot file path. Only `.sna` files are supported currently
    #[structopt(long, conflicts_with = "file-autodetect")]
    pub snap: Option<PathBuf>,
    /// Set screen file to load. Only `.scr` files are supported currently
    #[structopt(long, conflicts_with = "file-autodetect")]
    pub screen: Option<PathBuf>,

    /// Load provided file to emulator. Emulator will perform autodetect of format if possible
    pub file_autodetect: Option<PathBuf>,
}

fn machine_from_str(s: &str) -> Result<ZXMachine, anyhow::Error> {
    match s.to_lowercase().as_str() {
        "48k" | "48" => Ok(ZXMachine::Sinclair48K),
        "128k" | "128" => Ok(ZXMachine::Sinclair128K),
        s => Err(anyhow::anyhow!("Invalid machine type `{}`", s)),
    }
}

fn emualtion_speed_from_str(s: &str) -> Result<EmulationMode, anyhow::Error> {
    match s.to_lowercase().as_str() {
        "max" => Ok(EmulationMode::Max),
        s => {
            let speed: std::num::NonZeroUsize = s
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid emulation speed `{}`", s))?;

            Ok(EmulationMode::FrameCount(speed.into()))
        }
    }
}

fn scale_from_str(s: &str) -> Result<usize, anyhow::Error> {
    let scale: std::num::NonZeroUsize = s
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid screen scale `{}`", s))?;

    Ok(scale.into())
}

fn ay_mode_from_str(s: &str) -> Result<ZXAYMode, anyhow::Error> {
    match s.to_lowercase().as_str() {
        "mono" => Ok(ZXAYMode::Mono),
        "abc" => Ok(ZXAYMode::ABC),
        "acb" => Ok(ZXAYMode::ACB),
        s => Err(anyhow::anyhow!("Invalid AY chip mode `{}`", s)),
    }
}

fn sound_latency_from_str(s: &str) -> Result<usize, anyhow::Error> {
    let latency = s
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("Invalid sound latency `{}`", s))?;

    if latency < 64 {
        anyhow::bail!("Setting sound latency lower than 64 is bad for your health");
    }
    if latency > 1024 * 64 {
        anyhow::bail!("This sound latency is HUGE. Please don't try this at home!");
    }

    Ok(latency)
}

fn sound_sample_rate_from_str(s: &str) -> Result<usize, anyhow::Error> {
    let sample_rate = s
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("Invalid sound sample rate {}", s))?;

    // Sample rate range derived from https://github.com/audiojs/sample-rate
    if sample_rate < 8000 {
        anyhow::bail!("Provided sound sample rate `{}` is too low", sample_rate);
    }
    if sample_rate > 384000 {
        anyhow::bail!("Provided sound sample rate `{}` is too high", sample_rate);
    }

    Ok(sample_rate)
}

impl Settings {
    pub fn to_rustzx_settings(&self) -> RustzxSettings {
        let ay_enabled = (matches!(self.machine, ZXMachine::Sinclair128K) || self.force_enable_ay)
            && (!self.force_disable_ay);

        RustzxSettings {
            machine: self.machine,
            emulation_mode: self.speed,
            tape_fastload_enabled: !self.disable_fastload,
            kempston_enabled: !self.disable_kempston,
            mouse_enabled: self.enable_mouse,
            ay_mode: self.ay_mode,
            ay_enabled,
            beeper_enabled: !self.disable_beeper,
            sound_enabled: !self.disable_sound,
            sound_volume: 100,
            load_default_rom: self.rom.is_none(),
            sound_sample_rate: self.sound_sample_rate,
            autoload_enabled: !self.disable_autoload,
        }
    }
}
