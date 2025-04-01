use num_traits::{SaturatingAdd, SaturatingSub, Signed, Unsigned};
use rand::{distributions::uniform::SampleUniform, Rng};

/// Pick a random number between `lower_modifier` and `upper_modifier` for a given value
/// that also clamps between a `min` and `max`.
/// This is the signed version of this helper for f32, f64 and signed integer types.
pub fn random_inc_dec_clamp_signed<R, T>(
    rng: &mut R,
    value: T,
    lower_modifier: T,
    upper_modifier: T,
    min: T,
    max: T,
) -> T
where
    R: Rng,
    T: Copy + Signed + PartialOrd + SampleUniform,
{
    let lower = if value - lower_modifier < min {
        min
    } else {
        value - lower_modifier
    };
    let upper = if value + upper_modifier > max {
        max
    } else {
        value + upper_modifier
    };

    rng.gen_range(lower..=upper)
}

/// Pick a random number between `lower_modifier` and `upper_modifier` for a given value
/// that also clamps between a `min` and `max`.
/// This is the unsigned version of this helper for unsigned integer types.
pub fn random_inc_dec_clamp_unsigned<R, T>(
    rng: &mut R,
    value: T,
    lower_modifier: T,
    upper_modifier: T,
    min: T,
    max: T,
) -> T
where
    R: Rng,
    T: Copy + Unsigned + SaturatingAdd + SaturatingSub + PartialOrd + SampleUniform,
{
    let lower = if value.saturating_sub(&lower_modifier) < min {
        min
    } else {
        value.saturating_sub(&lower_modifier)
    };
    let upper = if value.saturating_add(&upper_modifier) > max {
        max
    } else {
        value.saturating_add(&upper_modifier)
    };

    rng.gen_range(lower..=upper)
}

/// If one in `how_many` chance happens, it will return true else returns false
pub fn one_chance_in_many<R>(rng: &mut R, how_many: u32) -> bool
where
    R: Rng,
{
    rng.gen_range(0..how_many) == 0
}
