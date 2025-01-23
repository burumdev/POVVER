use crate::{environment::Environment, speed::Speed, timer::Timer};

pub type SimInt = usize;
pub type SimFlo = f32;
pub type TickDuration = u64;

pub const DEFAULT_TICK_DURATION: TickDuration = 500;

#[derive(Debug)]
pub struct Simulation {
    timer: Timer,
    speed: Speed,
    env: Environment,
    ui: bool,
    entities: bool,
    is_running: bool,
    is_paused: bool,
}

impl Simulation {
    pub fn new() -> Self {
        let speed = Speed::NORMAL;
        let mut timer = Timer::new(speed.get_tick_duration(), 12);

        let timer_result = timer.tick();

        Self {
            timer,
            speed,
            env: Environment::new(timer_result),
            ui: true,
            entities: true,
            is_running: false,
            is_paused: false,
        }
    }
}

impl Simulation {
    pub fn run(&mut self) {
        self.is_running = true;

        loop {
            if !self.is_paused {
                let timer_result = self.timer.tick();
                self.env.update(timer_result);
            }

            if !self.is_running {
                break;
            }
        }
    }

    pub fn quit(&mut self) {
        self.is_running = false;
        println!("This simulation ended. Now your's continue.");
    }

    pub fn toggle_paused(&mut self) {
        self.is_paused = !self.is_paused;
    }
}
