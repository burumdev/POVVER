use crate::simulation::{SimFlo, TickDuration, DEFAULT_TICK_DURATION};

#[derive(Debug, Copy, Clone)]
pub struct Speed(pub TickDuration);

impl Speed {
    pub const SLOWEST: Self = Self(DEFAULT_TICK_DURATION * 4);
    pub const SLOW: Self = Self(DEFAULT_TICK_DURATION * 2);
    pub const SLOWER: Self = Self((DEFAULT_TICK_DURATION as SimFlo * 1.5) as TickDuration);
    pub const NORMAL: Self = Self(DEFAULT_TICK_DURATION);
    pub const FASTER: Self = Self(DEFAULT_TICK_DURATION / 8);
    pub const FAST: Self = Self(DEFAULT_TICK_DURATION / 16);
    pub const FASTEST: Self = Self(DEFAULT_TICK_DURATION / 32);
}

impl Default for Speed {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl Speed {
    pub fn get_tick_duration(&self) -> TickDuration {
        self.0
    }
}

pub const SPEEDS_ARRAY: [Speed; 7] = [
    Speed::SLOWEST,
    Speed::SLOW,
    Speed::SLOWER,
    Speed::NORMAL,
    Speed::FASTER,
    Speed::FAST,
    Speed::FASTEST
];
