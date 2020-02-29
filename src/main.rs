mod utils;
mod expression_tree;
mod gen_expr;
mod display_expr;

pub use expression_tree::{IExpr, VExpr, Unary, Binary};

fn main() {
    let mut rng = rand::thread_rng();
    let expr = gen_expr::Parameters::default().gen_expr(&mut rng, 5);
    println!("{}", expr);
}
