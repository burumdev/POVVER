use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use crate::{
    app_state::TimerState,
    simulation::{SimInt, TickDuration},
    ui_controller::Date
};
use crate::environment::months::get_month_data;

#[derive(Debug, PartialEq)]
pub enum TimerEvent {
    Paused,
    NothingUnusual,
    HourChange,
    DayChange,
    MonthChange,
    YearChange,
}

#[derive(Debug)]
pub struct Timer {
    tick_duration: TickDuration,
    tick_count: u128,
    timer_state: Arc<RwLock<TimerState>>,
}

// Constructor
impl Timer {
    pub fn new(tick_duration: TickDuration, init_date: Date) -> (Self, Arc<RwLock<TimerState>>) {
        let tick_count =
        (
            (
                (
                    init_date.hour +
                    ((init_date.day - 1) * 24) +
                    ((init_date.month - 1) * 30 * 24) +
                    (init_date.year * 12 * 30 * 24)
                ) * 60
            ) + init_date.minute
        ) as u128;

        let month_data = get_month_data(init_date.month as usize);
        let timer_state = Arc::new(RwLock::new(TimerState {
            date: init_date,
            month_data,
        }));

        (
            Self {
                tick_duration,
                tick_count,
                timer_state: Arc::clone(&timer_state),
            },
            timer_state,
        )
    }
}

// Private methods
impl Timer {
    fn get_updated_date(&self) -> Date {
        let minute = (self.tick_count % 60) as SimInt;

        let total_hours = self.tick_count / 60;
        let hour = (total_hours % 24) as SimInt;

        let total_days = total_hours / 24;
        let day = (total_days % 30) as SimInt + 1;

        let total_months = total_days / 30;
        let month = (total_months % 12) as SimInt + 1;

        let year = (total_months / 12) as SimInt;

        Date {
            minute,
            hour,
            day,
            month,
            year,
        }
    }
}

// Public API
impl Timer {
    pub fn tick(&mut self, is_paused: bool) -> TimerEvent {
        let mut event: TimerEvent;
        if !is_paused {
            thread::sleep(Duration::from_millis(self.tick_duration));

            self.tick_count = self.tick_count.wrapping_add(1);
            let date = self.get_updated_date();
            event = TimerEvent::NothingUnusual;

            let mut ts_lock = self.timer_state.write().unwrap();
            let prev_date = &ts_lock.date;

            if date.year != prev_date.year {
                event = TimerEvent::YearChange;
            } else if date.month != prev_date.month {
                ts_lock.month_data = get_month_data(date.month as usize);
                event = TimerEvent::MonthChange;
            } else if date.day != prev_date.day {
                event = TimerEvent::DayChange;
            } else if date.hour != prev_date.hour {
                event = TimerEvent::HourChange;
            }

            ts_lock.date = date;
        } else { // Paused
            thread::sleep(Duration::from_millis(500));
            event = TimerEvent::Paused;
        }

        event
    }

    pub fn set_tick_duration(&mut self, duration_ms: TickDuration) {
        self.tick_duration = duration_ms;
    }

    pub fn get_tick_duration(&self) -> TickDuration{
        self.tick_duration
    }
}
