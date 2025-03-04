use std::ops::{Div, Sub};
use crate::simulation::{SimFlo, SimInt};

pub trait Flippable {
    fn flip(&mut self) -> Self;
}

pub trait Percentage<'a>
where
    Self: 'a,
    &'a Self: Div<SimFlo, Output = SimFlo>,
{
    fn as_factor(&'a self) -> SimFlo {
        self / 100.0
    }
}

impl Percentage<'_> for SimFlo {}
