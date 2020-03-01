use rand::Rng;

use crate::expr::{IExpr, VExpr, Unary, Binary};
use crate::utils::{self, weighted_choice};

pub struct Parameters {
    root_iexpr_weights: [f32; 3],
    iexpr_weights: [f32; 7],
    max_depth_iexpr_weights: [f32; 3],
    vexpr_weights: [f32; 5],

    unary_weights: [f32; 8],
    binary_weights: [f32; 3],
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            root_iexpr_weights: [1.0; 3],
            max_depth_iexpr_weights: [1.0; 3],
            iexpr_weights: [1.0; 7],
            vexpr_weights: [1.0; 5],

            unary_weights: [1.0; 8],
            binary_weights: [1.0; 3],
        }
    }
}

impl Parameters {
    fn perturb<R: Rng>(&mut self, rng: &mut R) {
        utils::perturb(rng, &mut self.root_iexpr_weights);
        utils::perturb(rng, &mut self.iexpr_weights);
        utils::perturb(rng, &mut self.max_depth_iexpr_weights);
        utils::perturb(rng, &mut self.vexpr_weights);

        utils::perturb(rng, &mut self.unary_weights);
        utils::perturb(rng, &mut self.binary_weights);
    }

    fn gen_literal<R: Rng>(rng: &mut R) -> IExpr {
        if rng.gen() {
            IExpr::Rgb([
                utils::small_positive(rng),
                utils::small_positive(rng),
                utils::small_positive(rng),
            ])
        } else {
            IExpr::Lit(rng.gen())
        }
    }

    fn gen_unary<R: Rng>(&self, rng: &mut R) -> Unary {
        match weighted_choice(rng, &self.unary_weights) {
            0 => Unary::Square,
            1 => Unary::Cube,
            2 => Unary::Abs,
            3 => Unary::Neg,
            4 => Unary::DivBy(utils::small_positive(rng)),
            5 => Unary::ModBy(utils::small_positive(rng)),
            6 => Unary::Mod256,
            7 => Unary::Clamp256,
            _ => unreachable!(),
        }
    }

    fn gen_binary<R: Rng>(&self, rng: &mut R) -> Binary {
        match weighted_choice(rng, &self.binary_weights) {
            0 => Binary::Add,
            1 => Binary::Sub,
            2 => Binary::Mul,
            _ => unreachable!(),
        }
    }

    fn gen_vexpr<R: Rng>(&self, rng: &mut R, max_depth: u8) -> VExpr {
        if max_depth == 0 {
            VExpr::Pixel
        } else {
            match weighted_choice(rng, &self.vexpr_weights) {
                0 => VExpr::Pixel,
                1 => VExpr::Swap(Box::new(self.gen_vexpr(rng, max_depth - 1))),
                2 => VExpr::BinaryI(
                    self.gen_binary(rng),
                    self.gen_binary(rng),
                    Box::new(self.gen_iexpr(rng, max_depth - 1)),
                    Box::new(self.gen_iexpr(rng, max_depth - 1))),
                3 => VExpr::UnaryV(
                    self.gen_unary(rng),
                    Box::new(self.gen_vexpr(rng, max_depth - 1))),
                4 => VExpr::BinaryV(
                    self.gen_binary(rng),
                    Box::new(self.gen_vexpr(rng, max_depth - 1)),
                    Box::new(self.gen_vexpr(rng, max_depth - 1))),
                _ => unreachable!()
            }
        }
    }

    fn gen_iexpr<R: Rng>(&self, rng: &mut R, max_depth: u8) -> IExpr {
        if max_depth == 0 {
            match weighted_choice(rng, &self.max_depth_iexpr_weights) {
                0 => Self::gen_literal(rng),
                1 => if rng.gen() { IExpr::PixelX } else { IExpr::PixelY }
                2 => IExpr::Channel,
                _ => unreachable!()
            }
        } else {
            match weighted_choice(rng, &self.iexpr_weights) {
                0 => Self::gen_literal(rng),
                1 => if rng.gen() { IExpr::PixelX } else { IExpr::PixelY }
                2 => IExpr::Channel,
                3 => IExpr::Scale256(Box::new(self.gen_iexpr(rng, max_depth - 1))),
                4 => IExpr::UnaryI(
                    self.gen_unary(rng),
                    Box::new(self.gen_iexpr(rng, max_depth - 1))),
                5 => IExpr::BinaryI(
                    self.gen_binary(rng),
                    Box::new(self.gen_iexpr(rng, max_depth - 1)),
                    Box::new(self.gen_iexpr(rng, max_depth - 1))),
                6 => IExpr::BinaryV(
                    self.gen_binary(rng),
                    Box::new(self.gen_vexpr(rng, max_depth - 1))),
                _ => unreachable!(),
            }
        }
    }

    pub fn gen_expr<R: Rng>(&self, rng: &mut R, max_depth: u8) -> IExpr {
        let interior = self.gen_iexpr(rng, max_depth);
        match weighted_choice(rng, &self.root_iexpr_weights) {
            0 => IExpr::Scale256(Box::new(interior)),
            1 => IExpr::UnaryI(Unary::Mod256, Box::new(interior)),
            2 => IExpr::UnaryI(Unary::Clamp256, Box::new(interior)),
            _ => unreachable!(),
        }
    }
}