mod utils;

fn main() {
    let mut rng = rand::thread_rng();
    let mut weights = [0.1, 0.7, 0.2];
    let mut choices = [0, 0, 0];
    for _ in 0..1_000_000 {
        let idx = utils::weighted_choice(&mut rng, &weights);
        choices[idx] += 1;
    }
    println!("{:?}", choices);
}
