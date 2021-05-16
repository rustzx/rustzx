#![no_std]
mod backends;

pub use backends::AymPrecise;

use core::fmt::Debug;
use num_traits::Num;

/// AY/YM Sound chip register count
pub const AY_REGISTER_COUNT: usize = 14;

/// Samples, returned by `aym` implemnt this trait; It allows to correctly convert
/// the sound sample between different types. e.g.
/// [to_f32](AySample::to_f32)/[to_f64](AySample::to_f64) will always return sample in the
/// correct range `[-1.0; 1.0]`, while for example [to_i8](AySample::to_i8)
/// will return the sound sample in range `[-128; 127]`
pub trait AySample: Num + Copy {
    // Returns sound sample in range `[i8::MIN; i8::MAX]`
    fn to_i8(self) -> i8;
    // Returns sound sample in range `[i16::MIN; i16::MAX]`
    fn to_i16(self) -> i16;
    // Returns sound sample in range `[i32::MIN; i32::MAX]`
    fn to_i32(self) -> i32;
    // Returns sound sample in range `[-1.0; 1.0]`
    fn to_f32(self) -> f32;
    // Returns sound sample in range `[-1.0; 1.0]`
    fn to_f64(self) -> f64;
}

impl AySample for f64 {
    fn to_i8(self) -> i8 {
        (i8::MAX as f64 * self).clamp(i8::MIN as f64, i8::MAX as f64) as i8
    }

    fn to_i16(self) -> i16 {
        (i16::MAX as f64 * self).clamp(i16::MIN as f64, i16::MAX as f64) as i16
    }

    fn to_i32(self) -> i32 {
        (i32::MAX as f64 * self).clamp(i32::MIN as f64, i32::MAX as f64) as i32
    }

    fn to_f32(self) -> f32 {
        self as f32
    }

    fn to_f64(self) -> f64 {
        self
    }
}

/// Represents AY stereo sample with `left` and `right` channel samples
#[derive(Debug)]
pub struct StereoSample<S>
where
    S: AySample + Debug,
{
    pub left: S,
    pub right: S,
}

/// Sound chip type
#[derive(Debug)]
pub enum SoundChip {
    /// AY-3-8910 sound chip
    AY,
    /// YM2149 sound chip
    YM,
}

/// Stereo configuration
///
/// `Both` - Played on both channels
/// `Left` - Played only on left channel
/// `Right` - Played only on right channel
///
/// | Mode | A     | B     | C     |
/// | ---- | ----- | ----- | ----- |
/// | Mono | Both  | Both  | Both  |
/// | ABC  | Left  | Both  | Right |
/// | ACB  | Left  | Right | Both  |
/// | BAC  | Both  | Left  | Right |
/// | BCA  | Right | Left  | Both  |
/// | CAB  | Both  | Right | Left  |
/// | CBA  | Right | Both  | Left  |
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum AyMode {
    Mono,
    ABC,
    ACB,
    BAC,
    BCA,
    CAB,
    CBA,
}

/// Sound library generation backend.
///
/// Currently is only one backend - [AymPrecise],
pub trait AymBackend: Sized {
    /// Resulting sample type
    type SoundSample: AySample + Debug;

    /// Creates new aym instance.
    ///
    /// `frequency` - frequency of the sound chip in Hz
    /// `sample_rate` - target device sound sample rate
    fn new(chip: SoundChip, mode: AyMode, frequency: usize, sample_rate: usize) -> Self;
    /// Write value to the sound chip register. `address` should be in `[0..AY_REGISTER_COUNT]`
    fn write_register(&mut self, address: u8, value: u8);
    /// Generates next sound sample
    fn next_sample(&mut self) -> StereoSample<Self::SoundSample>;
}
