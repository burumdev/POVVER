use crate::simulation::{SimFlo, SimInt};

#[derive(Debug, Copy, Clone)]
pub enum CloudSize {
    Small,
    Normal,
    Big,
}
pub const CLOUD_SIZES: &[CloudSize] = &[CloudSize::Small, CloudSize::Normal, CloudSize::Big];

#[derive(Debug, Copy, Clone)]
pub struct Cloud {
    pub size: CloudSize,
    pub position: SimInt,
}

#[derive(Debug)]
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
impl SunBrightness {
    pub fn val(&self) -> SimFlo {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub enum WindDirection {
    Rtl,
    Ltr,
}
