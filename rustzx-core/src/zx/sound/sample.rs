use core::ops::{Add, Mul, MulAssign, Sub};

pub trait RawSample: Clone + Copy + MulAssign + Mul + Add + Sub {}
impl RawSample for f64 {}
impl RawSample for f32 {}
impl RawSample for i16 {}

// Sound sample type
// Have it's have two special cases: `SoundSample<f64>`
// And `SoundSample<i16>`
// `SoundSample<f64>` is using for audio processing.
// It have special functions:
// - `mix` - for mixing with another source
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
        SoundSample { left, right }
    }

    /// multiplies self 2 channels by `val`
    pub fn mul_eq(&mut self, val: T) -> &mut Self {
        self.left *= val;
        self.right *= val;
        self
    }

    /// multiplies self channels separately
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
        self.left += sample.left;
        self.right += sample.right;
        self
    }

    /// transform into f32
    pub fn into_f32(self) -> SoundSample<f32> {
        SoundSample {
            left: self.left as f32,
            right: self.right as f32,
        }
    }
}

/// Trait which signals that structure can generate SoundSamples
pub(crate) trait SampleGenerator<T>
where
    T: RawSample,
{
    /// Returns generated sound sample of `SoundSample<T>` type
    fn gen_sample(&mut self) -> SoundSample<T>;
}
