use rand::Rng;
use rand::distributions::{Open01, Uniform};

/// Choose an integer `n` from the range 0..weights.len() with probability
/// proportional to `weights[n]`.
pub fn weighted_choice<R: Rng>(rng: &mut R, weights: &[f64]) -> usize {
    let sum = weights.iter().sum::<f64>();
    let val = sum * rng.sample::<f64, Open01>(Open01);
    let mut prev_sum = 0.0;
    let mut i = 0;
    while { prev_sum += weights[i]; prev_sum < val } {
        i += 1;
    }
    return i;
}

/// Return a small positive number. This can be anywhere from 0 to 45 but in practice
/// will almost always be less than 15.
pub fn small_positive<R: Rng>(rng: &mut R) -> u8 {
    let dist = Uniform::new_inclusive(-2, 2);
    let mut sum = 0i8;
    for _ in 0..15 {
        sum += rng.sample(dist);
    }
    if sum == 0 {
        1
    } else {
        sum.abs() as u8
    }
}

pub fn clamp(min: i32, max: i32, n: i32) -> i32 {
    std::cmp::min(max, std::cmp::max(min, n))
}