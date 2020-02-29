use rand::SeedableRng;
use rand::rngs::StdRng;

mod utils;
mod expr;
mod gen_expr;
mod display_expr;

pub use expr::{IExpr, VExpr, Unary, Binary};

fn main() {
    let mut rng = StdRng::from_seed([3; 32]);
    let expr = gen_expr::Parameters::default().gen_expr(&mut rng, 5);
    println!("{}", expr);
}