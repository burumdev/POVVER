use super::SUNSHINE_MAX;
use crate::simulation::{SimFlo, SimInt};
use crate::ui_controller::{SunData, CloudData, SunStage as SlintSunStage};
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

impl Into<CloudData> for Cloud {
    fn into(self) -> CloudData {
        CloudData {
            size: self.size as i32,
            position: self.position as i32,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TheSun {
    pub position: i32,
    pub brightness: SunBrightness,
    pub stage: SunStage,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum SunStage {
    #[default]
    Set,
    Weak,
    Normal,
    Bright,
}

impl Into<SlintSunStage> for SunStage {
    fn into(self) -> SlintSunStage {
        match self {
            SunStage::Set => SlintSunStage::Set,
            SunStage::Weak => SlintSunStage::Weak,
            SunStage::Normal => SlintSunStage::Normal,
            SunStage::Bright => SlintSunStage::Bright,
        }
    }
}

impl Into<SunData> for TheSun {
    fn into(self) -> SunData {
        SunData {
            position: self.position,
            brightness: self.brightness.val(),
            stage: self.stage.into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
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

impl WindDirection {
    pub fn flip(&mut self) -> Self {
        if *self == Self::Rtl {
            *self = Self::Ltr;

            Self::Ltr
        } else {
            *self = Self::Rtl;

            Self::Rtl
        }
    }
}
