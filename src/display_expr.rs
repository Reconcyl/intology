use std::fmt::{self, Display, Formatter};

use crate::expr::{IExpr, VExpr, Unary, Binary};

impl Display for Unary {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Unary::*;
        match self {
            Square => write!(f, "square"),
            Cube => write!(f, "cube"),
            Abs => write!(f, "abs"),
            Neg => write!(f, "neg"),
            DivBy(n) => write!(f, "/{}", n),
            ModBy(n) => write!(f, "%{}", n),
            Mod256 => write!(f, "mod-256"),
            Clamp256 => write!(f, "clamp"),
        }
    }
}

impl Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Binary::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
        }
    }
}

impl Display for IExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use IExpr::*;
        match self {
            Lit(n) => write!(f, "{}", n),
            Rgb([r, g, b]) => write!(f, "{}/{}/{}", r, g, b),
            PixelX => write!(f, "x"),
            PixelY => write!(f, "y"),
            Channel => write!(f, "c"),
            Scale256(sub_e) => write!(f, "(scale-256 {})", sub_e),
            UnaryI(op, sub_e) => write!(f, "({} {})", op, sub_e),
            BinaryI(op, e_1, e_2) => write!(f, "({} {} {})", op, e_1, e_2),
            BinaryV(op, sub_e) => write!(f, "({} {})", op, sub_e),
        }
    }
}

impl Display for VExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use VExpr::*;
        match self {
            Pixel => write!(f, "xy"),
            Swap(sub_e) => write!(f, "[swap {}]", sub_e),
            BinaryI(op_1, op_2, e_1, e_2) => write!(f, "[[{} {}] {} {}]", op_1, op_2, e_1, e_2),
            UnaryV(op, sub_e) => write!(f, "[{} {}]", op, sub_e),
            BinaryV(op, e_1, e_2) => write!(f, "[{} {} {}]", op, e_1, e_2),
        }
    }
}