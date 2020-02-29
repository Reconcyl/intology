use rand::Rng;
use rand::distributions::Open01;

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