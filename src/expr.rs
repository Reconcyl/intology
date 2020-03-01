use serde::{Serialize, Deserialize};

use crate::utils::clamp;

type Color = [i32; 3];

/// An expression that returns a single 32-bit integer.
#[derive(Serialize, Deserialize)]
pub enum IExpr {
    Lit(i32),
    Rgb([u8; 3]),
    PixelX,
    PixelY,
    Channel,
    Scale256(Box<IExpr>),
    UnaryI(Unary, Box<IExpr>),
    BinaryI(Binary, Box<IExpr>, Box<IExpr>),
    BinaryV(Binary, Box<VExpr>),
    IfThenElseI(Box<IExpr>, Box<IExpr>, Box<IExpr>),
    IfThenElseV(Box<IExpr>, Box<VExpr>),
}

/// An expression that returns a pair of 32-bit integers.
#[derive(Serialize, Deserialize)]
pub enum VExpr {
    Pixel,
    Swap(Box<VExpr>),
    BinaryI(Binary, Binary, Box<IExpr>, Box<IExpr>),
    UnaryV(Unary, Box<VExpr>),
    BinaryV(Binary, Box<VExpr>, Box<VExpr>),
    IfThenElseI(Box<IExpr>, Box<VExpr>, Box<VExpr>),
    IfThenElseV(Box<VExpr>, Box<VExpr>, Box<VExpr>),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Unary {
    Square,
    Cube,
    Abs,
    Neg,
    DivBy(u8),
    ModBy(u8),
    Mod256,
    Clamp256,
}

impl Unary {
    fn eval(self, n: i32) -> i32 {
        use Unary::*;
        match self {
            Square => n.wrapping_mul(n),
            Cube => n.wrapping_mul(n).wrapping_mul(n),
            Abs => n.wrapping_abs(),
            Neg => n.wrapping_neg(),
            DivBy(d) => n.wrapping_div_euclid(d as i32),
            ModBy(d) => n.wrapping_rem_euclid(d as i32),
            Mod256 => n.wrapping_rem_euclid(256),
            Clamp256 => clamp(0, 255, n),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Binary {
    Add,
    Sub,
    Mul,
    BitAnd,
    BitOr,
    BitXor,
}

impl Binary {
    fn eval(self, a: i32, b: i32) -> i32 {
        use Binary::*;
        match self {
            Add => a.wrapping_add(b),
            Sub => a.wrapping_sub(b),
            Mul => a.wrapping_mul(b),
            BitAnd => a & b,
            BitOr => a | b,
            BitXor => a ^ b,
        }
    }
}

#[derive(Clone, Copy)]
enum StackItem {
    Same(i32),
    Rgb(i32, i32, i32),
}

impl StackItem {
    fn rgb(self) -> Color {
        use StackItem::*;
        match self {
            Same(n) => [n, n, n],
            Rgb(r, g, b) => [r, g, b]
        }
    }
    fn eval_unary(self, op: Unary) -> Self {
        use StackItem::*;
        match self {
            Same(n) => Same(op.eval(n)),
            Rgb(r, g, b) => Rgb(op.eval(r), op.eval(g), op.eval(b)),
        }
    }
    fn eval_binary(self, other: Self, op: Binary) -> Self {
        use StackItem::*;
        let eval = |a, b| op.eval(a, b);
        match (self, other) {
            (Same(n1), Same(n2)) => Same(eval(n1, n2)),
            (Same(n), Rgb(r, g, b)) =>
                Rgb(eval(n, r), eval(n, g), eval(n, b)),
            (Rgb(r, g, b), Same(n)) =>
                Rgb(eval(r, n), eval(g, n), eval(b, n)),
            (Rgb(r1, g1, b1), Rgb(r2, g2, b2)) =>
                Rgb(eval(r1, r2), eval(g1, g2), eval(b1, b2)),
        }
    }
}

impl From<i32> for StackItem {
    fn from(i: i32) -> Self {
        Self::Same(i)
    }
}

impl From<Color> for StackItem {
    fn from([r, g, b]: Color) -> Self {
        Self::Rgb(r, g, b)
    }
}

struct BatchEntry {
    pos: (i32, i32),
    stack: Vec<StackItem>,
}

impl BatchEntry {
    fn pop(&mut self) -> StackItem {
        self.stack.pop().unwrap()
    }
    fn pop_2(&mut self) -> (StackItem, StackItem) {
        let a = self.pop();
        let b = self.pop();
        (b, a)
    }
    fn get(&self, idx_from_back: usize) -> StackItem {
        let idx = self.len() - 1 - idx_from_back;
        self.stack[idx]
    }
    fn get_mut(&mut self, idx_from_back: usize) -> &mut StackItem {
        let idx = self.len() - 1 - idx_from_back;
        &mut self.stack[idx]
    }
    fn len(&self) -> usize {
        self.stack.len()
    }
    fn push(&mut self, item: impl Into<StackItem>) {
        self.stack.push(item.into());
    }
}

struct Batch {
    entries: Vec<BatchEntry>,
}

impl Batch {
    fn each<F: Fn(&mut BatchEntry)>(&mut self, f: F) {
        self.entries.iter_mut().for_each(f);
    }
    fn scale_256(&mut self, idx: usize) {
        let mut minimums = [std::i32::MAX; 3];
        let mut maximums = [std::i32::MIN; 3];
        for entry in &self.entries {
            let components = entry.get(idx).rgb();
            for ch in 0..3 {
                minimums[ch] = std::cmp::min(minimums[ch], components[ch]);
                maximums[ch] = std::cmp::max(maximums[ch], components[ch]);
            }
        }
        // We use f64s instead of f32s to prevent these subtractions from overflowing.
        let ranges = [
            maximums[0] as f64 - minimums[0] as f64,
            maximums[1] as f64 - minimums[1] as f64,
            maximums[2] as f64 - minimums[2] as f64,
        ];
        for entry in &mut self.entries {
            let item = entry.get_mut(idx);
            let mut components = item.rgb();
            for ch in 0..3 {
                let channel = &mut components[ch];
                let range = ranges[ch];
                let relative =
                    if range == 0.0 { 0.5 }
                    else { (*channel as f64 - minimums[ch] as f64) / range };
                if relative < 0.0 || relative > 1.0 {
                    println!("{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n\n",
                        minimums, maximums, ranges, channel, relative);
                }
                assert!(0.0 <= relative && relative <= 1.0);
                *channel = (255.0 * relative) as i32;
            }
            *item = components.into();
        }
    }
}

impl VExpr {
    fn eval_on_batch(&self, batch: &mut Batch) {
        use VExpr::*;
        match self {
            Pixel => batch.each(|e| {
                e.push(e.pos.0);
                e.push(e.pos.1);
            }),
            Swap(sub_e) => {
                sub_e.eval_on_batch(batch);
                batch.each(|e| {
                    let (a, b) = e.pop_2();
                    e.push(b);
                    e.push(a);
                })
            }
            BinaryI(op_1, op_2, e_1, e_2) => {
                e_1.eval_on_batch(batch);
                e_2.eval_on_batch(batch);
                batch.each(|e| {
                    let (a, b) = e.pop_2();
                    let new_1 = a.eval_binary(b, *op_1);
                    let new_2 = a.eval_binary(b, *op_2);
                    e.push(new_1);
                    e.push(new_2);
                })
            }
            UnaryV(op, sub_e) => {
                sub_e.eval_on_batch(batch);
                batch.each(|e| {
                    let (n_1, n_2) = e.pop_2();
                    let new_1 = n_1.eval_unary(*op);
                    let new_2 = n_2.eval_unary(*op);
                    e.push(new_1);
                    e.push(new_2);
                })
            }
            BinaryV(op, e_1, e_2) => {
                e_1.eval_on_batch(batch);
                e_2.eval_on_batch(batch);
                batch.each(|e| {
                    let (b_1, b_2) = e.pop_2();
                    let (a_1, a_2) = e.pop_2();
                    let new_1 = a_1.eval_binary(b_1, *op);
                    let new_2 = a_2.eval_binary(b_2, *op);
                    e.push(new_1);
                    e.push(new_2);
                })
            }
            IfThenElseI(e_cond, e_then, e_else) => {
                e_cond.eval_on_batch(batch);
                e_then.eval_on_batch(batch);
                e_else.eval_on_batch(batch);
                batch.each(|e| {
                    let (else_1, else_2) = e.pop_2();
                    let (then_1, then_2) = e.pop_2();
                    let cond = e.pop();
                    e.push(crate::utils::array_zip_3(
                        |c, t, e| if c > 0 { t } else { e },
                        cond.rgb(),
                        then_1.rgb(),
                        else_1.rgb(),
                    ));
                    e.push(crate::utils::array_zip_3(
                        |c, t, e| if c > 0 { t } else { e },
                        cond.rgb(),
                        then_2.rgb(),
                        else_2.rgb(),
                    ));
                })
            }
            IfThenElseV(e_cond, e_then, e_else) => {
                e_cond.eval_on_batch(batch);
                e_then.eval_on_batch(batch);
                e_else.eval_on_batch(batch);
                batch.each(|e| {
                    let (else_1, else_2) = e.pop_2();
                    let (then_1, then_2) = e.pop_2();
                    let (cond_1, cond_2) = e.pop_2();
                    e.push(crate::utils::array_zip_3(
                        |c, t, e| if c > 0 { t } else { e },
                        cond_1.rgb(),
                        then_1.rgb(),
                        else_1.rgb(),
                    ));
                    e.push(crate::utils::array_zip_3(
                        |c, t, e| if c > 0 { t } else { e },
                        cond_2.rgb(),
                        then_2.rgb(),
                        else_2.rgb(),
                    ));
                })
            }
        }
    }
}

impl IExpr {
    fn eval_on_batch(&self, batch: &mut Batch) {
        use IExpr::*;
        match self {
            Lit(n) => batch.each(|e| e.push(*n)),
            Rgb([r, g, b]) => batch.each(|e| e.push([*r as i32, *g as i32, *b as i32])),
            PixelX => batch.each(|e| e.push(e.pos.0)),
            PixelY => batch.each(|e| e.push(e.pos.1)),
            Channel => batch.each(|e| e.push([-1, 0, 1])),
            Scale256(sub_e) => {
                sub_e.eval_on_batch(batch);
                batch.scale_256(0);
            }
            UnaryI(op, sub_e) => {
                sub_e.eval_on_batch(batch);
                batch.each(|e| {
                    let new_val = e.pop().eval_unary(*op);
                    e.push(new_val);
                });
            }
            BinaryI(op, e_1, e_2) => {
                e_1.eval_on_batch(batch);
                e_2.eval_on_batch(batch);
                batch.each(|e| {
                    let (a, b) = e.pop_2();
                    let new_val = a.eval_binary(b, *op);
                    e.push(new_val);
                });
            }
            BinaryV(op, sub_e) => {
                sub_e.eval_on_batch(batch);
                batch.each(|e| {
                    let (a, b) = e.pop_2();
                    let new_val = a.eval_binary(b, *op);
                    e.push(new_val)
                });
            }
            IfThenElseI(e_cond, e_then, e_else) => {
                e_cond.eval_on_batch(batch);
                e_then.eval_on_batch(batch);
                e_else.eval_on_batch(batch);
                batch.each(|e| {
                    let (val_then, val_else) = e.pop_2();
                    let val_cond = e.pop();
                    let new_val = crate::utils::array_zip_3(
                        |c, t, e| if c > 0 { t } else { e },
                        val_cond.rgb(),
                        val_then.rgb(),
                        val_else.rgb(),
                    );
                    e.push(new_val);
                })
            }
            IfThenElseV(e_cond, e_case) => {
                e_cond.eval_on_batch(batch);
                e_case.eval_on_batch(batch);
                batch.each(|e| {
                    let (val_then, val_else) = e.pop_2();
                    let val_cond = e.pop();
                    let new_val = crate::utils::array_zip_3(
                        |c, t, e| if c > 0 { t } else { e },
                        val_cond.rgb(),
                        val_then.rgb(),
                        val_else.rgb(),
                    );
                    e.push(new_val);
                })
            }
        }
    }
    pub fn eval_batch<PI>(&self, inputs: PI) -> impl Iterator<Item=Color>
    where
        PI: Iterator<Item=(i32, i32)>,
    {
        let mut batch = Batch {
            entries: inputs.map(|pos| BatchEntry { pos, stack: Vec::new() }).collect()
        };

        self.eval_on_batch(&mut batch);

        batch.entries.into_iter().map(|mut entry| {
            assert_eq!(entry.len(), 1);
            entry.pop().rgb()
        })
    }
}