use rand::Rng;
use rand::distributions::{Open01, Uniform};

fn rand_unit<R: Rng>(rng: &mut R) -> f32 {
    rng.sample::<f32, _>(Open01)
}

pub fn mutate<R: Rng>(rng: &mut R, p1: &[f32], p2: &[f32], out: &mut [f32]) {
    assert!(p1.len() == p2.len());
    assert!(p1.len() == out.len());

    for i in 0..p1.len() {
        out[i] = if rng.gen() { p1 } else { p2 }[i];
    }
}

/// Choose an integer `n` from the range 0..weights.len() with probability
/// proportional to `weights[n]`.
pub fn weighted_choice<R: Rng>(rng: &mut R, weights: &[f32]) -> usize {
    let sum = weights.iter().sum::<f32>();
    let val = sum * rand_unit(rng);
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

/// Adjust random entries of `values` by random amounts.
pub fn perturb<R: Rng>(rng: &mut R, values: &mut [f32]) {
    for val in values {
        // Determine whether to update the value at all
        if rng.gen_ratio(1, 4) {
            let change = 1.0 + 0.3 * rand_unit(rng) * rand_unit(rng);
            if rng.gen() {
                *val *= change;
            } else {
                *val /= change;
            }
        }
    }
}

pub fn clamp(min: i32, max: i32, n: i32) -> i32 {
    std::cmp::min(max, std::cmp::max(min, n))
}