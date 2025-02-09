use super::SUNSHINE_MAX;
use crate::simulation::SimFlo;
use crate::ui_controller::{SunData, SunStage, WindDirection};

#[derive(Debug, Default, Copy, Clone)]
pub struct TheSun {
    pub position: i32,
    pub brightness: SunBrightness,
    pub stage: SunStage,
}

impl Into<SunData> for TheSun {
    fn into(self) -> SunData {
        SunData {
            position: self.position,
            brightness: self.brightness.val(),
            stage: self.stage,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SunBrightness(SimFlo);
impl SunBrightness {
    pub const NONE: Self = Self(0.0);
    pub const WEAK: Self = Self(20.0);
    pub const NORMAL: Self = Self(50.0);
    pub const STRONG: Self = Self(75.0);
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

impl WindDirection {
    pub fn flip(&mut self) -> Self {
        match self {
            Self::Rtl => {
                *self = Self::Ltr;

                Self::Ltr
            },
            Self::Ltr => {
                *self = Self::Rtl;

                Self::Rtl
            },
        }
    }
}
