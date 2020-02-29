mod utils;
mod expression_tree;

pub use expression_tree::{IExpr, VExpr, Unary, Binary};

fn main() {
    let formula = IExpr::Scale256(Box::new(IExpr::PixelX));
    let test_values = [(0, 0), (1, 1), (4, 4)];
    println!("{:?}", formula.eval_batch(test_values.iter().copied()).collect::<Vec<_>>());
}
