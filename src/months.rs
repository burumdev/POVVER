use crate::simulation::SimInt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Month {
    pub number: SimInt,
    pub day_start: SimInt,
    pub day_end: SimInt,
    pub name: &'static str,
    pub sunshine_factor: f32,
    pub windspeed_factor: f32,
}

impl Default for Month {
    fn default() -> Self {
        MONTHS[0]
    }
}

impl Month {
    pub fn get_day_start_end(&self) -> (SimInt, SimInt) {
        (self.day_start, self.day_end)
    }
}

pub fn get_month(index: usize) -> &'static Month {
    &MONTHS[index]
}

pub const MONTHS: [Month; 12] = [
    Month {
        number: 1,
        name: "January",
        day_start: 8,
        day_end: 17,
        sunshine_factor: 0.5,
        windspeed_factor: 1.0,
    },
    Month {
        number: 2,
        name: "February",
        day_start: 8,
        day_end: 18,
        sunshine_factor: 0.5,
        windspeed_factor: 1.1,
    },
    Month {
        number: 3,
        name: "March",
        day_start: 7,
        day_end: 18,
        sunshine_factor: 0.65,
        windspeed_factor: 1.0,
    },
    Month {
        number: 4,
        name: "April",
        day_start: 6,
        day_end: 18,
        sunshine_factor: 0.75,
        windspeed_factor: 0.8,
    },
    Month {
        number: 5,
        name: "May",
        day_start: 6,
        day_end: 18,
        sunshine_factor: 1.0,
        windspeed_factor: 0.75,
    },
    Month {
        number: 6,
        name: "June",
        day_start: 6,
        day_end: 19,
        sunshine_factor: 1.25,
        windspeed_factor: 0.75,
    },
    Month {
        number: 7,
        name: "July",
        day_start: 5,
        day_end: 19,
        sunshine_factor: 1.5,
        windspeed_factor: 0.9,
    },
    Month {
        number: 8,
        name: "August",
        day_start: 6,
        day_end: 19,
        sunshine_factor: 1.5,
        windspeed_factor: 1.2,
    },
    Month {
        number: 9,
        name: "September",
        day_start: 7,
        day_end: 19,
        sunshine_factor: 1.1,
        windspeed_factor: 1.2,
    },
    Month {
        number: 10,
        name: "October",
        day_start: 7,
        day_end: 18,
        sunshine_factor: 0.75,
        windspeed_factor: 1.5,
    },
    Month {
        number: 11,
        name: "November",
        day_start: 7,
        day_end: 18,
        sunshine_factor: 0.65,
        windspeed_factor: 1.2,
    },
    Month {
        number: 12,
        name: "December",
        day_start: 8,
        day_end: 18,
        sunshine_factor: 0.5,
        windspeed_factor: 1.0,
    },
];
