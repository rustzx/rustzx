use std::i16;
use std::ops::{Add, Mul, MulAssign, Sub};
/// Raw Sample can be only f64 or i16
pub trait RawSample: Clone + Copy + MulAssign + Mul + Add + Sub {}
impl RawSample for f64 {}
impl RawSample for f32 {}
impl RawSample for i16 {}

const ERROR_SIZE: u16 = 100;
// Sound sample type
// Have it's have two special cases: `SoundSample<f64>`
// And `SoundSample<i16>`
// `SoundSample<f64>` is using for audio processing.
// It have special functions:
// - `mix` - for mixing with another source
// - `normalize` - to fit sample in 0..1 range
// - `into_i16` - to transform sa,ple to i16 sample
#[derive(Clone, Copy)]
pub struct SoundSample<T>
where
    T: RawSample,
{
    pub left: T,
    pub right: T,
}

impl<T> SoundSample<T>
where
    T: RawSample,
{
    /// Returns new sample
    pub fn new(left: T, right: T) -> SoundSample<T> {
        SoundSample {
            left: left,
            right: right,
        }
    }
    /// multiplies self 2 channels by `val`
    pub fn mul_eq(&mut self, val: T) -> &mut Self {
        self.left *= val;
        self.right *= val;
        self
    }
    /// multiplies self channels separatly
    pub fn mul(&mut self, val_left: T, val_right: T) -> &mut Self {
        self.left *= val_left;
        self.right *= val_right;
        self
    }
}
/// Trait specialization for float `RawSample`
impl SoundSample<f64> {
    /// Mixes self with another sample
    pub fn mix<'a>(&'a mut self, sample: &SoundSample<f64>) -> &'a mut Self {
        self.left = self.left + sample.left - self.left * sample.left;
        self.right = self.right + sample.right - self.right * sample.right;
        self
    }
    /// Transforms normalized float sample to i16 sample
    pub fn into_i16(self) -> SoundSample<i16> {
        // here is some thick hack :D
        // we have value in 0.0..1.0. We need value in min_i16...max_i16 (signed)
        // so we unly need one float multiplication, one floor and one XOR with highest bit
        // multiplication + floor => 0...max_u16
        // XOR =>  4 bit example {
        //      0b0000 [0] => 0b1000 [-8]
        //      0b0001 [1] => 0b1000 [-7]
        //      ...
        //      0b0111 [7] => 0b1111 [-1]
        //      0b1000 [8] => 0b0000 [0]
        //      0b1000 [9] => 0b0001 [1]
        //      ...
        //      0b1111 [15] => 0b0111 [7]
        // }
        // So we can easily get range expansion only with XOR operation in MSb
        // NOTE: `i16 as u16` and `u16 as i16` have no cost
        SoundSample {
            left: ((self.left * (u16::max_value() - ERROR_SIZE) as f64) as u16 ^ 0x8000) as i16,
            right: ((self.right * (u16::max_value() - ERROR_SIZE) as f64) as u16 ^ 0x8000) as i16,
        }
    }
    /// transform into f32
    pub fn into_f32(self) -> SoundSample<f32> {
        SoundSample {
            left: self.left as f32,
            right: self.right as f32,
        }
    }
    /// Places float sample in range 0..1
    /// # Arguments
    /// - `min` - minimal original value
    /// - `max` - maximal original value
    /// # Example
    /// If original value in range -1...1 then `normalize(-1.0, 1.0)`
    /// will transform it to 0.0..1.0 range
    pub fn normalize(&mut self, min: f64, max: f64) -> &mut Self {
        self.left = (self.left - min) / (max - min);
        self.right = (self.right - min) / (max - min);
        self
    }
}

/// Trait which signals that structure can generate SoundSamples
pub trait SampleGenerator<T>
where
    T: RawSample,
{
    /// Returns generated sound sample of `SoundSample<T>` type
    fn gen_sample(&mut self) -> SoundSample<T>;
}
