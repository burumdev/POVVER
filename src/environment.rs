use std::ops::Add;

use rand::{random, rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

use crate::environment_types::{
    Cloud, CloudSize, SunBrightness, TheSun, WindDirection, CLOUD_SIZES,
};
use crate::months::MonthData;
use crate::simulation::{SimFlo, SimInt};
use crate::timer::{TimerEvent, TimerPayload};

const WINDSPEED_MAX: SimInt = 120;
const CLOUDS_MAX: SimInt = 16;
const SUN_POS_MAX: SimInt = 16;

#[derive(Debug)]
pub struct Environment {
    clouds: Vec<Cloud>,
    wind_speed: SimInt,
    wind_direction: WindDirection,
    the_sun: TheSun,
    rng: ThreadRng,
}

// Constructor
impl Environment {
    pub fn new(timer_payload: TimerPayload) -> Self {
        let mut rng = thread_rng();

        let mut clouds = Vec::with_capacity(CLOUDS_MAX);

        let cloud_generate_count =
            rng.gen_range(0..CLOUDS_MAX) as SimFlo * timer_payload.month_data.cloud_forming_factor;

        for _ in 0..cloud_generate_count as SimInt {
            let size = *CLOUD_SIZES.choose(&mut rng).unwrap();
            let position = rng.gen_range(0..CLOUDS_MAX);
            clouds.push(Cloud { size, position });
        }

        let wind_speed = rng.gen_range(0..=WINDSPEED_MAX);
        let wind_speed = (wind_speed as f32 * timer_payload.month_data.windspeed_factor) as SimInt;

        let wind_direction = if random() {
            WindDirection::Ltr
        } else {
            WindDirection::Rtl
        };

        let the_sun = Self::get_the_sun(timer_payload.date.hour, timer_payload.month_data);

        Self {
            clouds,
            wind_speed,
            wind_direction,
            the_sun,
            rng,
        }
    }
}

// Private methods
impl Environment {
    fn get_the_sun(hour: SimInt, month: &MonthData) -> TheSun {
        let (start, end) = month.get_day_start_end();

        // It's still night out there...
        if hour < start || hour > end {
            return TheSun {
                position: None,
                brightness: SunBrightness::NONE,
            };
        }

        // It's daytime. Let's create the sun!
        let float_hour = hour as SimFlo;
        let total_day_hours = end - start;
        let unit = (total_day_hours as SimFlo) / 12.0;
        let mid_unit = unit * 6.0;

        // Dead middle of the day
        let mid_point = start as SimFlo + mid_unit;

        let brightness: SunBrightness;
        // Strong sunshine is 2 units wide, mid is 10 units wide and weak is 1 units wide on each end from a total of 12
        if ((mid_point - unit)..=(mid_point + unit)).contains(&float_hour) {
            brightness = SunBrightness::STRONG;
        } else if ((mid_point - (unit * 5.0))..=(mid_point + (unit * 5.0))).contains(&float_hour) {
            brightness = SunBrightness::NORMAL;
        } else {
            brightness = SunBrightness::WEAK;
        }

        let sunrise_shift = ((SUN_POS_MAX - total_day_hours) / 2) as SimFlo;
        let position = Some(hour - start + (sunrise_shift.ceil() as SimInt));

        TheSun {
            position,
            brightness,
        }
    }

    fn maybe_new_cloud(
        &mut self,
        tail_pos: SimInt,
        sibling_pos: SimInt,
        cloud_forming_factor: SimFlo,
    ) -> Option<Cloud> {
        let tail_clouds_count = self
            .clouds
            .iter()
            .filter(|cloud| cloud.position == tail_pos)
            .count();

        if tail_clouds_count < 2 {
            let siblings = self
                .clouds
                .iter()
                .filter(|cloud| cloud.position == sibling_pos);

            let probability = siblings.fold(10.0, |acc, cloud| match cloud.size {
                CloudSize::Small => acc + 2.0,
                CloudSize::Normal => acc + 5.0,
                CloudSize::Big => acc + 10.0,
            }) * cloud_forming_factor;

            if self.rng.gen_range(0..=100) <= probability as SimInt {
                return Some(Cloud {
                    size: *CLOUD_SIZES.choose(&mut self.rng).unwrap(),
                    position: tail_pos,
                });
            } else {
                return None;
            }
        }

        None
    }

    fn update_clouds(&mut self, cloud_forming_factor: SimFlo) {
        if self.wind_speed < 10 {
            return;
        }

        let mut tail_pos = 0;
        let mut sibling_pos = 1;

        self.clouds.retain_mut(|cloud| {
            let Cloud { size, position } = cloud;

            let mut movement: f32 = match size {
                CloudSize::Small => 3.0,
                CloudSize::Normal => 2.0,
                CloudSize::Big => 1.0,
            };

            movement = match self.wind_speed {
                0..40 => movement / 3.0,
                40..80 => movement / 2.0,
                80..=WINDSPEED_MAX => movement,
                _ => unreachable!(),
            };

            let movement = movement.round() as SimInt;

            if self.wind_direction == WindDirection::Rtl {
                tail_pos = CLOUDS_MAX - 1;
                sibling_pos = CLOUDS_MAX - 2;
                let subtractable_position = *position as isize;
                if (subtractable_position - movement as isize) < 0 {
                    false
                } else {
                    *position -= movement as SimInt;

                    true
                }
            } else {
                if *position + movement > CLOUDS_MAX {
                    false
                } else {
                    *position += movement as SimInt;

                    true
                }
            }
        });

        if let Some(cloud) = self.maybe_new_cloud(tail_pos, sibling_pos, cloud_forming_factor) {
            self.clouds.push(cloud);
        }
    }
}

// Public API
impl Environment {
    pub fn update(&mut self, timer_payload: TimerPayload) {
        // We don't change stuff too often to prevent erratic changes
        // so the changes are done on an hourly basis.
        if timer_payload.event != TimerEvent::NothingUnusual {
            let hour = timer_payload.date.hour;
            let month_data = timer_payload.month_data;

            // Every 2nd hour we update wind speed
            // not deviating much from the current one.
            if hour % 2 == 0 {
                // Make tropical typhoons that last weeks less likely
                let lower_modifier = if self.wind_speed >= WINDSPEED_MAX - (WINDSPEED_MAX / 6) {
                    20
                } else {
                    0
                };
                let ws_lower = self.wind_speed.saturating_sub(5 + lower_modifier);
                let ws_upper = self.wind_speed.add(5).clamp(0, WINDSPEED_MAX);

                let wind_speed = (self.rng.gen_range(ws_lower..ws_upper) as f32
                    * month_data.windspeed_factor) as SimInt;

                self.wind_speed = wind_speed.clamp(0, WINDSPEED_MAX);
            }

            // Every 6th hour there is a 1 in 10 chance
            // the wind direction will change
            // but only if it's sufficiently weak currently.
            if hour % 6 == 0 && self.wind_speed < 20 {
                let change_wind_direction = self.rng.gen_range(1..=10) == 10;
                if change_wind_direction {
                    self.wind_direction = if self.wind_direction == WindDirection::Ltr {
                        WindDirection::Rtl
                    } else {
                        WindDirection::Ltr
                    };
                }
            }

            self.update_clouds(month_data.cloud_forming_factor);

            self.the_sun = Self::get_the_sun(timer_payload.date.hour, month_data);
        }

        if timer_payload.event != TimerEvent::NothingUnusual {
            println!(
                "TIMER: hour: {}, month: {}",
                timer_payload.date.hour, timer_payload.month_data.name
            );
            println!(
                "ENV: sun: {:?}, windspeed: {}, wind direction: {:?}",
                self.the_sun, self.wind_speed, self.wind_direction
            );
            println!("CLOUDS: {:?}", self.clouds);
        }
    }
}
