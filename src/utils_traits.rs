use num_traits::FromPrimitive;
use crate::simulation::SimFlo;

pub trait Flippable {
    fn flip(&mut self) -> Self;
}

pub trait AsFactor {
    fn as_factor(&self) -> SimFlo {
        self.val() / 100.0
    }
    fn val(&self) -> SimFlo;
}
pub trait HundredPercentable: AsFactor
where
    Self: FromPrimitive,
{
    const PERCENT_MAX: SimFlo = 100.0;
    fn percent_clamp(&self, val: SimFlo) -> SimFlo {
        val.clamp(0.0, Self::PERCENT_MAX)
    }
    fn set(&mut self, val: SimFlo) {
        *self = Self::from_f32(self.percent_clamp(val)).unwrap();
    }
}

impl AsFactor for SimFlo {
    fn val(&self) -> SimFlo {
        *self
    }
}
