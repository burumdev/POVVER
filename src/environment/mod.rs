use std::sync::{Arc, RwLock};
use rand::{random, rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

mod environment_types;
pub use environment_types::*;

use crate::{
    app_state::{TimerState, EnvState},
    ui_controller::{Cloud, CloudSize, SunStage, WindDirection, WindSpeedLevel},
    months::Month,
    simulation::{SimFlo, SimInt},
    utils_random::{one_chance_in_many, random_inc_dec_clamp_signed},
    utils_traits::Flippable,
};

pub const WINDSPEED_MAX: SimInt = 120;
const CLOUD_POS_MAX: SimInt = 15;
pub const CLOUDS_MAX: SimInt = 32;
const SUN_POS_MAX: SimInt = 15;
pub const SUNSHINE_MAX: SimFlo = 100.0;

#[derive(Debug)]
pub struct Environment {
    env_state: Arc<RwLock<EnvState>>,
    timer_state: Arc<RwLock<TimerState>>,
    rng: ThreadRng,
}

// Constructor
impl Environment {
    pub fn new(timer_state: Arc<RwLock<TimerState>>) -> (Self, Arc<RwLock<EnvState>>) {
        let mut rng = thread_rng();

        let mut clouds = Vec::with_capacity(CLOUDS_MAX as usize);
        let mut wind_speed = WindSpeed::default();

        {
            let ts_lock = timer_state.read().unwrap();
            let month_data = ts_lock.month_data;

            let cloud_generate_count = rng.gen_range(0..CLOUD_POS_MAX) as SimFlo
                * month_data.cloud_forming_factor;

            for _ in 0..cloud_generate_count as SimInt {
                let size = *CLOUD_SIZES.choose(&mut rng).unwrap();
                let position = rng.gen_range(0..CLOUD_POS_MAX);
                let image_index = rng.gen_range(0..4);
                clouds.push(Cloud {
                    size,
                    position,
                    image_index,
                    image_rotated: random(),
                });
            }

            wind_speed.set(
                (rng.gen_range(0..=WINDSPEED_MAX) as SimFlo * month_data.windspeed_factor) as SimInt
            );
        };

        let wind_direction = if random() {
            WindDirection::Ltr
        } else {
            WindDirection::Rtl
        };

        let env_state = Arc::new(RwLock::new(EnvState {
            clouds,
            wind_speed,
            wind_direction,
            the_sun: TheSun::default(),
            sunshine_reduction: 0.0,
        }));

        (
            Self {
                env_state: Arc::clone(&env_state),
                timer_state,
                rng,
            },
            env_state
        )
    }
}

// Private methods
impl Environment {
    fn get_the_sun(&self, clouds: &[Cloud], hour: SimInt, month: &Month) -> TheSun {
        let (start, end) = month.get_day_start_end();

        match hour {
            // It's still night out there...
            h if h < start => TheSun {
                position: 17,
                brightness: SunBrightness::NONE,
                stage: SunStage::Set,
                brightness_reduction: 0.0,
            },
            h if h > end => TheSun {
                position: -1,
                brightness: SunBrightness::NONE,
                stage: SunStage::Set,
                brightness_reduction: 0.0,
            },
            // Sun is rising!
            _ => {
                let float_hour = hour as SimFlo;
                let total_day_hours = end - start;

                /*
                    POSITION OF THE SUN
                 */
                // Nudge the sunrise position to the right depending on season.
                // Sun rises late in winter and early in summer.
                // And then reposition the sun so it can do it's thing.
                let sunrise_shift = ((SUN_POS_MAX + 1 - total_day_hours) / 2) as SimFlo;
                // Find the position of the sun incremental from left to right (west to east)
                // Not saturated_sub'ing because this should not fail
                // given the night check at the start of this function works.
                let position = ((hour - start) as f32 + sunrise_shift.ceil()) as i32;
                // Now invert the position of the sun so it
                // rises from the east and sets from the west.
                let position = SUN_POS_MAX as i32 - position;

                /*
                    SUN BRIGHTNESS
                 */
                let unit = (total_day_hours as SimFlo) / 14.0;
                let mid_unit = unit * 7.0;

                // Dead middle of the day in units
                let mid_point = start as SimFlo + mid_unit;

                let mut brightness: SunBrightness;
                let stage: SunStage;
                // Strong sunshine is 3 units wide, mid is 9 units wide and weak is 2 unit wide
                // on each end (middle out like in pied piper) from a total of 14.
                // So it's like WWMMMMMSSSMMMMMWW in unit terms
                // And when we turn it to integers at the most it should look like:
                // WMMMSMMMW in winter and something like WWMMMMSSSMMMMWW in a hot summer day.
                if ((mid_point - unit * 1.5)..=(mid_point + unit * 1.5)).contains(&float_hour) {
                    brightness = SunBrightness::STRONG;
                    stage = SunStage::Bright;
                } else if ((mid_point - (unit * 6.0))..=(mid_point + (unit * 6.0))).contains(&float_hour) {
                    brightness = SunBrightness::NORMAL;
                    stage = SunStage::Normal;
                } else {
                    brightness = SunBrightness::WEAK;
                    stage = SunStage::Weak;
                }
                // Different months have varying degrees of sunshine
                brightness.set(brightness.val() * month.sunshine_factor);
                // If any clouds cover the sun, firstly shame on them
                // and secondly they should cumulatively reduce the
                // brightness of sun depending on their sizes.
                let brightness_reduction = clouds
                    .iter()
                    .filter(|cloud| cloud.position == position)
                    .fold(0.0, |acc, cloud| match cloud.size {
                        CloudSize::Small => acc + 5.0,
                        CloudSize::Medium => acc + 15.0,
                        CloudSize::Big => acc + 25.0,
                    });
                if brightness_reduction > 0.0 {
                    brightness.set(brightness.val() - brightness_reduction);
                    println!(
                        "REDUCING SUNSHINE! by {} and new brightness is {:?}",
                        brightness_reduction, brightness
                    );
                }

                TheSun {
                    position,
                    brightness,
                    brightness_reduction,
                    stage,
                }
            }
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
        &self,
        clouds: &[Cloud],
        wind_speed: &WindSpeed,
        tail_pos: SimInt,
        sibling_pos: SimInt,
        cloud_forming_factor: SimFlo,
    ) -> Option<Cloud> {
        let mut rng = thread_rng();

        let tail_clouds_count = clouds
            .iter()
            .filter(|cloud| cloud.position == tail_pos)
            .count();

        if tail_clouds_count > 1 {
            let siblings = clouds
                .iter()
                .filter(|cloud| cloud.position == sibling_pos);

            let probability = siblings.fold(10.0, |acc, cloud| match cloud.size {
                CloudSize::Small => acc + 2.0,
                CloudSize::Medium => acc + 5.0,
                CloudSize::Big => acc + 10.0,
            }) * cloud_forming_factor;

            let wind_speed_rnd_ceiling = (match WindSpeedLevel::from(wind_speed) {
                WindSpeedLevel::Faint => 120,
                WindSpeedLevel::Mild => 100,
                WindSpeedLevel::Strong => 80,
                WindSpeedLevel::Typhoon => 60,
            } as SimFlo * cloud_forming_factor) as SimInt;

            if rng.gen_range(0..=wind_speed_rnd_ceiling) <= probability as SimInt {
                Some(Cloud {
                    size: *CLOUD_SIZES.choose(&mut rng).unwrap(),
                    position: tail_pos,
                    image_index: rng.gen_range(0..4),
                    image_rotated: random(),
                })
            } else {
                None
            }
        } else {
            let wind_speed_rnd_how_many = (match WindSpeedLevel::from(wind_speed) {
                WindSpeedLevel::Faint => 60,
                WindSpeedLevel::Mild => 30,
                WindSpeedLevel::Strong => 20,
                WindSpeedLevel::Typhoon => 10,
            } as SimFlo * cloud_forming_factor) as u32;
            if one_chance_in_many(&mut rng, wind_speed_rnd_how_many) {
                Some(Cloud {
                    size: *CLOUD_SIZES.choose(&mut rng).unwrap(),
                    position: tail_pos,
                    image_index: rng.gen_range(0..4),
                    image_rotated: random(),
                })
            } else {
                None
            }
        }
    }

    // This mainly moves the clouds and
    // removes clouds that exit the scene from left or right.
    // Also maybe adds another cloud.
    fn update_clouds(&mut self, cloud_forming_factor: SimFlo) {
        let (wind_speed, wind_direction) = {
            let es_lock = self.env_state.read().unwrap();

            (
                es_lock.wind_speed.clone(),
                es_lock.wind_direction,
            )
        };

        let mut env = self.env_state.write().unwrap();

        env.clouds.retain_mut(|cloud| {
            let Cloud { size, position, .. } = cloud;

            let mut movement: SimFlo = match size {
                CloudSize::Small => 3.0,
                CloudSize::Medium => 2.0,
                CloudSize::Big => 1.5,
            };

            movement = match WindSpeedLevel::from(&wind_speed) {
                WindSpeedLevel::Faint => movement / 5.0,
                WindSpeedLevel::Mild => movement / 3.0,
                WindSpeedLevel::Strong => movement / 2.0,
                WindSpeedLevel::Typhoon => movement,
            };

            let movement = movement.round() as SimInt;

            if wind_direction == WindDirection::Rtl {
                if (*position - movement) < 0 {
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

        if env.clouds.len() < CLOUDS_MAX as usize {
            let tail_pos =
                if wind_direction == WindDirection::Rtl { CLOUD_POS_MAX } else { 0 };
            let sibling_pos =
                if wind_direction == WindDirection::Rtl { CLOUD_POS_MAX - 1 } else { 1 };

            if let Some(cloud) = self.maybe_new_cloud(env.clouds.as_slice(), &wind_speed, tail_pos, sibling_pos, cloud_forming_factor) {
                println!("PUSHING IN A NEW CLOUD: {:?}", cloud);
                env.clouds.push(cloud);
            }
        }
    }
}

// Public API
impl Environment {
    pub fn update(&mut self) {
        // We don't change stuff too often to prevent erratic changes
        // so the changes are done on an hourly basis.
        let (hour, month_data) = {
            let ts_lock = self.timer_state.read().unwrap();

            (ts_lock.date.hour, ts_lock.month_data)
        };

        {
            let mut es_lock = self.env_state.write().unwrap();

            // Every 6th hour there is a 1 in 10 chance
            // the wind direction will change
            // but only if it's sufficiently weak currently.
            // Otherwise, set the wind_speed every two hours.
            if hour % 6 == 0 && es_lock.wind_speed < 40 && one_chance_in_many(&mut self.rng, 10) {
                es_lock.wind_direction.flip();
                es_lock.wind_speed.set((10.0 * month_data.windspeed_factor) as SimInt);
            } else if hour % 2 == 0 {
                // Make tropical typhoons that last weeks less likely
                let ws_val = es_lock.wind_speed.val();
                let lower_modifier = match ws_val {
                    ws if ws >= WINDSPEED_MAX - 40 => 40,
                    ws if ws >= WINDSPEED_MAX - 50 => 30,
                    ws if ws >= WINDSPEED_MAX - 60 => 20,
                    _ => 0,
                };

                let randomized = random_inc_dec_clamp_signed(
                    &mut self.rng,
                    ws_val,
                    lower_modifier + 10,
                    5,
                    0,
                    WINDSPEED_MAX,
                );

                es_lock.wind_speed.set((randomized as SimFlo * month_data.windspeed_factor) as SimInt);
            }
        }

        self.update_clouds(month_data.cloud_forming_factor);

        {
            let mut es_lock = self.env_state.write().unwrap();
            es_lock.the_sun = self.get_the_sun(es_lock.clouds.as_slice(), hour, month_data);

            println!("---------- HOUR CHANGE ----------");
            println!("CLOUDS: {:?}", es_lock.clouds);
        }
    }
}
