use rand::{
    Rng,
    distributions::uniform::SampleUniform,
};

use std::ops::{Add, Sub};

pub fn random_inc_dec_clamped<R, T>(rng: &mut R, value: T, modifier: T, min: T, max: T) -> T
where
    R: Rng,
    T: SampleUniform + Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let mut lower = value - modifier;
    if lower < min {
        lower = min;
    }
    let mut upper = value + modifier;
    if upper > max {
        upper = max;
    }

   rng.gen_range(lower..upper)
}