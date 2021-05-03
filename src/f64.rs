use serde::ser::{Serialize, Serializer};
use std::ops::{Deref, DerefMut};

/// An `f64` wrapper that ensures only 4 digits after the the decimal point.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct F64(f64);

impl From<f64> for F64 {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Deref for F64 {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for F64 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for F64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = (self.0 * 1_0000.0).round() / 1_0000.0;

        serializer.serialize_f64(value)
    }
}
