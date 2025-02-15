use super::{SUNSHINE_MAX, WINDSPEED_MAX};
use crate::{
    simulation::{SimFlo, SimInt},
    ui_controller::{CloudSize, SunData, SunStage, WindDirection, WindSpeedLevel},
    utils_traits::Flippable,
};

pub const CLOUD_SIZES: &[CloudSize] = &[CloudSize::Small, CloudSize::Medium, CloudSize::Big];

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
impl PartialEq<SimFlo> for SunBrightness {
    fn eq(&self, other: &SimFlo) -> bool {
        self.0.eq(other)
    }
}
impl PartialOrd<SimFlo> for SunBrightness {
    fn partial_cmp(&self, other: &SimFlo) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

#[derive(Debug, Default)]
pub struct WindSpeed(SimInt);
impl WindSpeed {
    pub fn val(&self) -> SimInt {
        self.0
    }
    pub fn set(&mut self, val: SimInt) {
        self.0 = val.clamp(0, WINDSPEED_MAX);
    }
}
impl PartialEq<SimInt> for WindSpeed {
    fn eq(&self, other: &SimInt) -> bool {
        self.0.eq(other)
    }
}
impl PartialOrd<SimInt> for WindSpeed {
    fn partial_cmp(&self, other: &SimInt) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
impl From<&WindSpeed> for WindSpeedLevel {
    fn from(ws: &WindSpeed) -> Self {
        match ws.val() {
            0..10 => WindSpeedLevel::Faint,
            10..40 => WindSpeedLevel::Mild,
            40..80 => WindSpeedLevel::Strong,
            80..=WINDSPEED_MAX => WindSpeedLevel::Typhoon,
            // Should be unrachable because we properly clamp the windspeed (hopefully)
            _ => unreachable!(),
        }
    }
}

impl Flippable for WindDirection {
    fn flip(&mut self) -> Self {
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
