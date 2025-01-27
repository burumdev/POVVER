use std::thread;
use std::time::Duration;

use crate::months::{get_month_data, MonthData};
use crate::simulation::{SimInt, TickDuration};

#[derive(Debug, Default, Copy, Clone)]
pub struct Date {
    pub minute: SimInt,
    pub hour: SimInt,
    pub day: SimInt,
    pub month: SimInt,
    pub year: SimInt,
}

#[derive(Debug, PartialEq)]
pub enum TimerEvent {
    NothingUnusual,
    DayChange,
    HourChange,
    MonthChange,
    YearChange,
}

#[derive(Debug)]
pub struct TimerPayload {
    pub date: Date,
    pub event: TimerEvent,
    pub month_data: &'static MonthData,
}

#[derive(Debug)]
pub struct Timer {
    tick_duration: TickDuration,
    tick_count: u128,
    date: Date,
}

// Constructor
impl Timer {
    pub fn new(tick_duration: TickDuration, init_hours: SimInt) -> Self {
        let init_hours = init_hours.clamp(0, 23);

        Self {
            tick_duration,
            tick_count: (init_hours * 60) as u128,
            date: Date {
                hour: init_hours,
                ..Date::default()
            },
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
        let month = (total_months % 12) as usize + 1;

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
    pub fn tick(&mut self) -> TimerPayload {
        //thread::sleep(Duration::from_millis(self.tick_duration));
        thread::sleep(Duration::from_millis(5));

        self.tick_count = self.tick_count.wrapping_add(1);

        let mut event = TimerEvent::NothingUnusual;

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

        TimerPayload {
            date: self.date,
            month_data: get_month_data(date.month),
            event,
        }
    }

    pub fn set_tick_duration(&mut self, duration_ms: TickDuration) {
        self.tick_duration = duration_ms;
    }
}
