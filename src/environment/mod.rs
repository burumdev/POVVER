use std::ops::Add;

use rand::{random, rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

mod environment_types;
mod economy;

use economy::Economy;
use environment_types::{Cloud, CloudSize, SunBrightness, TheSun, WindDirection};
use crate::months::MonthData;
use crate::simulation::{SimFlo, SimInt};
use crate::timer::{TimerEvent, TimerPayload};

use crate::utils::{one_chance_in_many, random_inc_dec_clamp_unsigned};

const WINDSPEED_MAX: SimInt = 120;
const CLOUD_POS_MAX: SimInt = 15;
const CLOUDS_MAX: SimInt = 32;
const SUN_POS_MAX: SimInt = 15;
pub const SUNSHINE_MAX: SimFlo = 150.0;

const CLOUD_SIZES: &[CloudSize] = &[CloudSize::Small, CloudSize::Normal, CloudSize::Big];

#[derive(Debug)]
pub struct Environment {
    economy: Economy,
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

        let mut economy = Economy::new();

        let mut clouds = Vec::with_capacity(CLOUDS_MAX);

        let cloud_generate_count = rng.gen_range(0..CLOUD_POS_MAX) as SimFlo
            * timer_payload.month_data.cloud_forming_factor;

        for _ in 0..cloud_generate_count as SimInt {
            let size = *CLOUD_SIZES.choose(&mut rng).unwrap();
            let position = rng.gen_range(0..CLOUD_POS_MAX);
            clouds.push(Cloud { size, position });
        }

        let wind_speed = rng.gen_range(0..=WINDSPEED_MAX);
        let wind_speed = (wind_speed as f32 * timer_payload.month_data.windspeed_factor) as SimInt;

        let wind_direction = if random() {
            WindDirection::Ltr
        } else {
            WindDirection::Rtl
        };

        let mut new_self = Self {
            economy,
            clouds,
            wind_speed,
            wind_direction,
            the_sun: TheSun::default(),
            rng,
        };

        new_self.the_sun = new_self.get_the_sun(timer_payload.date.hour, timer_payload.month_data);

        new_self
    }
}

// Private methods
impl Environment {
    fn get_the_sun(&self, hour: SimInt, month: &MonthData) -> TheSun {
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

        let mut brightness: SunBrightness;
        // Strong sunshine is 2 units wide, mid is 8 units wide and weak is 1 unit wide
        // on each end (middle out like in pied piper) from a total of 12.
        // So it's like WMMMMSSMMMMW in unit terms
        // And when we turn it to integers at the most it should look like:
        // WMMMSMMMW in winter and something like WWMMMMSSSMMMMWW in a hot summer day.
        if ((mid_point - unit)..=(mid_point + unit)).contains(&float_hour) {
            brightness = SunBrightness::STRONG;
        } else if ((mid_point - (unit * 5.0))..=(mid_point + (unit * 5.0))).contains(&float_hour) {
            brightness = SunBrightness::NORMAL;
        } else {
            brightness = SunBrightness::WEAK;
        }

        // Different months have varying degrees of sunshine
        brightness.set(brightness.val() * month.sunshine_factor);

        // If any clouds cover the sun, firstly shame on them
        // and secondly they should cumulatively reduce the
        // brightness of sun depending on their sizes.
        if let Some(position) = self.the_sun.position {
            let brightness_reduction = self
                .clouds
                .iter()
                .filter(|cloud| cloud.position == position)
                .fold(0.0, |acc, cloud| match cloud.size {
                    CloudSize::Small => acc + 5.0,
                    CloudSize::Normal => acc + 15.0,
                    CloudSize::Big => acc + 25.0,
                });

            if brightness_reduction > 0.0 {
                brightness.set(brightness.val() - brightness_reduction);
                println!(
                    "REDUCING SUNSHINE! by {} and new brightness is {:?}",
                    brightness_reduction, brightness
                );
            }
        }

        // Nudge the sunrise position to the right depending on season.
        // Sun rises late in winter and early in summer.
        // And then reposition the sun so it can do it's thing.
        let sunrise_shift = ((SUN_POS_MAX + 1 - total_day_hours) / 2) as SimFlo;
        // Not saturated_sub'ing because this should not fail
        // given the check at the start of this function works and nothing else is faulty.
        let position = Some(hour - start + (sunrise_shift.ceil() as SimInt));

        TheSun {
            position,
            brightness,
        }
    }

    // New cloud generator.
    // Position of the new cloud depends on the wind direction.
    // If tail_pos and sibling_pos have some clouds already,
    // the chance of generation is greater.
    // This is a simple way to model
    // natural-like cloud migrations that follow each other
    // and form clusters of clouds.
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

    // This mainly moves the clouds and
    // removes clouds that exit the scene from left or right.
    // Also maybe adds another cloud.
    fn update_clouds(&mut self, cloud_forming_factor: SimFlo) {
        if self.wind_speed <= 5 {
            return;
        }

        let mut tail_pos = 0;
        let mut sibling_pos = 1;
        if self.wind_direction == WindDirection::Rtl {
            tail_pos = CLOUD_POS_MAX;
            sibling_pos = CLOUD_POS_MAX - 1;
        }

        self.clouds.retain_mut(|cloud| {
            let Cloud { size, position } = cloud;

            let mut movement: SimFlo = match size {
                CloudSize::Small => 3.0,
                CloudSize::Normal => 2.0,
                CloudSize::Big => 1.0,
            };

            movement = match self.wind_speed {
                0..40 => movement / 3.0,
                40..80 => movement / 2.0,
                80..=WINDSPEED_MAX => movement,
                _ => unreachable!(), // Should be unreachable because we clamp the windspeed (hopefully)
            };

            let movement = movement.round() as SimInt;

            if self.wind_direction == WindDirection::Rtl {
                let subtractable_position = *position as isize;
                if (subtractable_position - movement as isize) < 0 {
                    false
                } else {
                    *position -= movement;

                    true
                }
            } else {
                if *position + movement > CLOUD_POS_MAX {
                    false
                } else {
                    *position += movement;

                    true
                }
            }
        });

        if self.clouds.len() < CLOUDS_MAX {
            if let Some(cloud) = self.maybe_new_cloud(tail_pos, sibling_pos, cloud_forming_factor) {
                println!("PUSHING IN A NEW CLOUD: {:?}", cloud);
                self.clouds.push(cloud);
            }
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

                self.wind_speed =
                    random_inc_dec_clamp_unsigned(&mut self.rng, self.wind_speed, lower_modifier + 5, 5, 0, WINDSPEED_MAX)
                        * month_data.windspeed_factor as SimInt;
            }

            // Every 6th hour there is a 1 in 10 chance
            // the wind direction will change
            // but only if it's sufficiently weak currently.
            if hour % 6 == 0 && self.wind_speed < 20 && one_chance_in_many(&mut self.rng, 10) {
                self.wind_direction.flip();
            }

            self.update_clouds(month_data.cloud_forming_factor);

            self.the_sun = self.get_the_sun(timer_payload.date.hour, month_data);

            println!("TIMER: {:?}", timer_payload.date);
            println!(
                "ENV: sun: {:?}, windspeed: {}, wind direction: {:?}",
                self.the_sun, self.wind_speed, self.wind_direction
            );
            println!("CLOUDS: {:?}", self.clouds);
        }

        if timer_payload.event == TimerEvent::MonthChange {
            self.economy.update();
            println!("ECONOMY: {:?}", self.economy);
        }
    }
}
