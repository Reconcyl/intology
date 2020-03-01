use rand::SeedableRng;
use rand::rngs::StdRng;

mod utils;
mod expr;
mod gen_expr;
mod display_expr;
mod gen_png;

pub use expr::{IExpr, VExpr, Unary, Binary};

fn main() {
    let seed = rand::random();
    let mut rng = StdRng::seed_from_u64(seed);
    let expr = gen_expr::Parameters::default().gen_expr(&mut rng, 5);
    let filename = format!("{}.png", seed);
    let file = std::fs::File::create(filename).unwrap();
    expr.write_image_data(file, 256, 256, 4);
    println!("Seed: {:?}", seed);
    println!("Expr: {}", expr);
}