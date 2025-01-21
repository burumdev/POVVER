use rand::{random, rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

use crate::months::Month;
use crate::simulation::{SimFlo, SimInt};
use crate::timer::{TimerEvent, TimerPayload};

const WIND_SPEED_MAX: SimInt = 100;
const CLOUDS_MAX: SimInt = 32;
const CLOUD_POSITION_MAX: SimInt = 64;

#[derive(Debug)]
enum WindDirection {
    Rtl,
    Ltr,
}

struct SunshineConst(SimFlo);
impl SunshineConst {
    pub const NONE: Self = Self(0.0);
    pub const WEAK: Self = Self(30.0);
    pub const NORMAL: Self = Self(80.0);
    pub const STRONG: Self = Self(130.0);
}
impl SunshineConst {
    fn val(&self) -> SimFlo {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
enum CloudSize {
    Small,
    Normal,
    Big,
}
const CLOUD_SIZE_VARIANTS: &[CloudSize] = &[CloudSize::Small, CloudSize::Normal, CloudSize::Big];

#[derive(Debug)]
struct Cloud {
    position: SimInt,
    size: CloudSize,
}

#[derive(Debug)]
pub struct Environment {
    clouds: Vec<Cloud>,
    wind_speed: SimInt,
    wind_direction: WindDirection,
    sunshine: SimFlo,
    rng: ThreadRng,
}

// Constructor
impl Environment {
    pub fn new(timer_payload: TimerPayload) -> Self {
        let mut rng = thread_rng();

        let mut clouds = Vec::with_capacity(CLOUDS_MAX);
        let cloud_generate_count = rng.gen_range(0..CLOUDS_MAX);
        for _ in 0..cloud_generate_count {
            let position = rng.gen_range(1..=CLOUD_POSITION_MAX);
            let size = *CLOUD_SIZE_VARIANTS.choose(&mut rng).unwrap();
            clouds.push(Cloud { position, size })
        }

        let wind_speed = rng.gen_range(0..WIND_SPEED_MAX);
        let wind_direction = if random() {
            WindDirection::Ltr
        } else {
            WindDirection::Rtl
        };

        let sunshine = Self::get_sunshine(timer_payload.date.hour, timer_payload.date.month);

        Self {
            clouds,
            wind_speed,
            wind_direction,
            sunshine,
            rng,
        }
    }

    fn get_sunshine(hour: SimInt, month: Month) -> SimFlo {
        let (start, end) = month.get_day_start_end();

        // It's still night out there...
        if hour < start || hour > end {
            return SunshineConst::NONE.val();
        }

        let float_hour = hour as SimFlo;
        let total_day_hours = end - start;
        let unit = (total_day_hours as SimFlo) / 12.0;
        let mid_unit = unit * 6.0;

        // Dead middle of the day
        let mid_point = start as SimFlo + mid_unit;

        let sunshine: SunshineConst;
        // Strong sunshine is 2 units wide, mid is 10 units wide and weak is 1 units wide on each end from a total of 12
        if ((mid_point - unit)..=(mid_point + unit)).contains(&float_hour) {
            sunshine = SunshineConst::STRONG;
        } else if ((mid_point - (unit * 5.0))..=(mid_point + (unit * 5.0))).contains(&float_hour) {
            sunshine = SunshineConst::NORMAL;
        } else {
            sunshine = SunshineConst::WEAK;
        }

        sunshine.val() * month.sunshine_factor
    }
}

// Private methods
impl Environment {}

// Public API
impl Environment {
    pub fn update(&mut self, timer_payload: TimerPayload) {
        let sunshine = Self::get_sunshine(timer_payload.date.hour, timer_payload.date.month);
        self.sunshine = sunshine;

        if timer_payload.event != TimerEvent::NothingUnusual {
            println!(
                "TIMER: hour: {}, month: {}",
                timer_payload.date.hour, timer_payload.date.month.name
            );
            println!("ENV: sunshine: {}", self.sunshine);
        }
    }
}
