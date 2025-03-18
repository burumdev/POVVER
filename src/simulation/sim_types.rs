use std::ops::{Add, MulAssign};
use num_traits::FromPrimitive;
use crate::utils_traits::{AsFactor, Percentable};

pub type SimInt = i32;
pub type SimFlo = f32;
pub type TickDuration = u64;

#[derive(Debug, Copy, Clone)]
pub struct Percentage(SimFlo);
impl Default for Percentage {
    fn default() -> Self {
        Percentage(0.0)
    }
}
impl Percentage {
    pub fn new(val: SimFlo) -> Self {
        Self(val)
    }
}

impl AsFactor for Percentage {
    fn val(&self) -> SimFlo {
        self.0
    }
}
impl FromPrimitive for Percentage {
    fn from_i64(val: i64) -> Option<Self> {
        Some(Self(val as SimFlo))
    }
    fn from_u64(val: u64) -> Option<Self> {
        Some(Self(val as SimFlo))
    }
    fn from_f32(val: f32) -> Option<Self> {
        Some(Self(val))
    }
}
impl Percentable for Percentage {}
impl PartialEq<SimFlo> for Percentage {
    fn eq(&self, other: &SimFlo) -> bool {
        self.0.eq(other)
    }
}
impl PartialOrd<SimFlo> for Percentage {
    fn partial_cmp(&self, other: &SimFlo) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
impl Add<SimFlo> for Percentage {
    type Output = SimFlo;
    fn add(self, other: SimFlo) -> SimFlo {
        self.percent_clamp(self.0 + other)
    }
}
impl MulAssign<SimFlo> for Percentage {
    fn mul_assign(&mut self, other: SimFlo) {
        self.0 = self.percent_clamp(self.0 * other)
    }
}
