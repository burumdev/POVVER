use crate::simulation::{TickDuration, DEFAULT_TICK_DURATION};

#[derive(Debug)]
pub struct Speed(pub TickDuration);

impl Speed {
    pub const FASTEST: Self = Self(DEFAULT_TICK_DURATION / 10);
    pub const FAST: Self = Self(DEFAULT_TICK_DURATION / 5);
    pub const FASTER: Self = Self(DEFAULT_TICK_DURATION / 2);
    pub const NORMAL: Self = Self(DEFAULT_TICK_DURATION);
    pub const SLOWER: Self = Self((DEFAULT_TICK_DURATION as f64 * 1.5) as TickDuration);
    pub const SLOW: Self = Self(DEFAULT_TICK_DURATION * 2);
    pub const SLOWEST: Self = Self(DEFAULT_TICK_DURATION * 4);
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
