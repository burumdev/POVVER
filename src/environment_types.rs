use crate::environment::SUNSHINE_MAX;
use crate::simulation::{SimFlo, SimInt};

#[derive(Debug, Copy, Clone)]
pub enum CloudSize {
    Small,
    Normal,
    Big,
}

#[derive(Debug, Copy, Clone)]
pub struct Cloud {
    pub size: CloudSize,
    pub position: SimInt,
}

#[derive(Debug, Default)]
pub struct TheSun {
    pub position: Option<SimInt>,
    pub brightness: SunBrightness,
}

#[derive(Debug)]
pub struct SunBrightness(SimFlo);
impl SunBrightness {
    pub const NONE: Self = Self(0.0);
    pub const WEAK: Self = Self(30.0);
    pub const NORMAL: Self = Self(70.0);
    pub const STRONG: Self = Self(100.0);
}
impl Default for SunBrightness {
    fn default() -> Self {
        Self::NONE
    }
}
impl SunBrightness {
    pub fn val(&self) -> SimFlo {
        self.0
    }
    pub fn set(&mut self, val: SimFlo) {
        self.0 = val.clamp(0.0, SUNSHINE_MAX);
    }
}

#[derive(Debug, PartialEq)]
pub enum WindDirection {
    Rtl,
    Ltr,
}
