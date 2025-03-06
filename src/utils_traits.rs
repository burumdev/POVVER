use std::ops::Div;
use crate::simulation::SimFlo;

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
