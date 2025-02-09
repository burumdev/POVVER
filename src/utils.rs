use num_traits::{SaturatingAdd, SaturatingSub, Signed, Unsigned};
use rand::{distributions::uniform::SampleUniform, Rng};

/// Pick a random number between `lower_end` and `upper_end` that also clamps between a `min` and `max`.
/// This is the signed version of this helper for f32, f64 and signed integer types.
pub fn random_inc_dec_clamp_signed<R, T>(
    rng: &mut R,
    value: T,
    lower_end: T,
    upper_end: T,
    min: T,
    max: T,
) -> T
where
    R: Rng,
    T: Copy + Signed + PartialOrd + SampleUniform,
{
    let lower = if value - lower_end < min {
        min
    } else {
        value - lower_end
    };
    let upper = if value + upper_end > max {
        max
    } else {
        value + upper_end
    };

    rng.gen_range(lower..upper)
}

/// Pick a random number between `lower_end` and `upper_end` that also clamps between a `min` and `max`.
/// This is the unsigned version of this helper for unsigned integer types.
pub fn random_inc_dec_clamp_unsigned<R, T>(
    rng: &mut R,
    value: T,
    lower_end: T,
    upper_end: T,
    min: T,
    max: T,
) -> T
where
    R: Rng,
    T: Copy + Unsigned + SaturatingAdd + SaturatingSub + PartialOrd + SampleUniform,
{
    let lower = if value.saturating_sub(&lower_end) < min {
        min
    } else {
        value.saturating_sub(&lower_end)
    };
    let upper = if value.saturating_add(&upper_end) > max {
        max
    } else {
        value.saturating_add(&upper_end)
    };

    rng.gen_range(lower..upper)
}

/// If one in `how_many` chance happens, it will return true else returns false
pub fn one_chance_in_many<R>(rng: &mut R, how_many: u32) -> bool
where
    R: Rng,
{
    rng.gen_range(0..how_many) == 0
}
