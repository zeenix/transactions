use serde::Serialize;
use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};

/// An `f64` wrapper that ensures only 4 digits after the the decimal point.
#[derive(Copy, Clone, Debug, Default, Serialize, PartialEq)]
pub struct F64(f64);

impl From<f64> for F64 {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Add for F64 {
    type Output = F64;

    fn add(self, rhs: F64) -> Self::Output {
        (((self.0 + rhs.0) * 1_0000.0).round() / 1_0000.0).into()
    }
}

impl Sub for F64 {
    type Output = F64;

    fn sub(self, rhs: F64) -> Self::Output {
        (((self.0 - rhs.0) * 1_0000.0).round() / 1_0000.0).into()
    }
}

impl AddAssign for F64 {
    fn add_assign(&mut self, other: F64) {
        self.0 = ((self.0 + other.0) * 1_0000.0).round() / 1_0000.0;
    }
}

impl SubAssign for F64 {
    fn sub_assign(&mut self, other: F64) {
        self.0 = ((self.0 - other.0) * 1_0000.0).round() / 1_0000.0;
    }
}

impl Deref for F64 {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
