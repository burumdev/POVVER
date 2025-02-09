use std::thread;
use std::time::Duration;

use crate::ui_controller::Date;

use crate::months::{get_month_data, MonthData};
use crate::simulation::{SimInt, TickDuration};

#[derive(Debug, PartialEq)]
pub enum TimerEvent {
    Paused,
    NothingUnusual,
    DayChange,
    HourChange,
    MonthChange,
    YearChange,
}

#[derive(Debug)]
pub struct TimerPayload {
    pub event: TimerEvent,
    pub month_data: &'static MonthData,
}

#[derive(Debug)]
pub struct Timer {
    tick_duration: TickDuration,
    tick_count: u128,
    pub date: Date,
}

// Constructor
impl Timer {
    pub fn new(tick_duration: TickDuration, init_date: Date) -> Self {
        Self {
            tick_duration,
            tick_count:
                (init_date.minute +
                    init_date.hour * 60 +
                    init_date.day * 24 * 60 +
                    init_date.month * 30 * 24 * 60 +
                    init_date.year * 12 * 30 * 24 * 60
                ) as u128,
            date: init_date,
        }
    }
}

// Private methods
impl Timer {
    fn get_updated_date(&self) -> Date {
        let minute = (self.tick_count % 60) as SimInt;

        let total_hours = self.tick_count / 60;
        let hour = (total_hours % 24) as SimInt;

        let total_days = total_hours / 24;
        let day = ((total_days % 30) + 1) as SimInt;

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
    pub fn tick(&mut self, is_paused: bool) -> TimerPayload {
        thread::sleep(Duration::from_millis(self.tick_duration));

        let mut event: TimerEvent;
        if !is_paused {
            self.tick_count = self.tick_count.wrapping_add(1);

            event = TimerEvent::NothingUnusual;
            let date = self.get_updated_date();
            if date.year != self.date.year {
                event = TimerEvent::YearChange;
            } else if date.month != self.date.month {
                event = TimerEvent::MonthChange;
            } else if date.day != self.date.day {
                event = TimerEvent::DayChange;
            } else if date.hour != self.date.hour {
                event = TimerEvent::HourChange;
            }

            self.date = date;
        } else { // Paused
            event = TimerEvent::Paused;
        }

        TimerPayload {
            month_data: get_month_data(self.date.month as usize),
            event,
        }
    }

    pub fn set_tick_duration(&mut self, duration_ms: TickDuration) {
        self.tick_duration = duration_ms;
    }

    pub fn get_tick_duration(&self) -> TickDuration{
        self.tick_duration
    }
}
