use std::ops::{AddAssign, Add, SubAssign, Sub};
/// Clocks count
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Clocks(pub usize);

impl Add for Clocks {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Clocks(self.0 + rhs.0)
    }
}

impl Add<usize> for Clocks {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        Clocks(self.0 + rhs)
    }
}

impl AddAssign for Clocks {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<usize> for Clocks {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub for Clocks {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Clocks(self.0 - rhs.0)
    }
}

impl Sub<usize> for Clocks {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self {
        Clocks(self.0 - rhs)
    }
}

impl SubAssign for Clocks {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl SubAssign<usize> for Clocks {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl Clocks {
    /// returns inner `usize` value
    pub fn count(&self) -> usize {
        self.0
    }
}
