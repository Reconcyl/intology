use std::collections::HashMap;

mod utils;

fn main() {
    let mut rng = rand::thread_rng();
    let mut c = [0u32; 256];

    for _ in 0..1_000_000 {
        c[utils::small_positive(&mut rng) as usize] += 1;
    }

    println!("{:?}", c.iter().enumerate().filter(|&(_, &n)| n != 0).collect::<Vec<_>>());
}
