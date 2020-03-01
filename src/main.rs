use rand::Rng;

use rouille::{Request, Response, router, try_or_400};

use std::sync::Mutex;

mod utils;
mod expr;
mod gen_expr;
mod display_expr;
mod gen_png;

use gen_expr::Parameters;

#[derive(Debug)]
struct ParamPoolEntry {
    upvotes: usize,
    downvotes: usize,
    params: Parameters,
}

impl ParamPoolEntry {
    fn score(&self) -> f64 {
        let vote_sum = self.upvotes + self.downvotes;
        self.upvotes as f64 / vote_sum as f64
    }
}

struct ParamPool {
    entries: Vec<ParamPoolEntry>,
}

impl ParamPool {
    fn new() -> Self {
        Self { entries: vec![
            ParamPoolEntry {
                upvotes: 1,
                downvotes: 1,
                params: Parameters::default(),
            }
        ] }
    }
    fn get_low_voted_idx<R: Rng>(&self, rng: &mut R) -> usize {
        let mut indices = (0..self.entries.len()).collect::<Vec<_>>();
        // Find the highest-scoring entries
        indices.sort_by(|&i, &j| self.entries[i].score().partial_cmp(&self.entries[j].score()).unwrap());
        let bound = 1 + rng.gen_range(0, indices.len());
        indices[rng.gen_range(0, bound)]
    }
    fn get_high_voted_idx<R: Rng>(&self, rng: &mut R) -> usize {
        let mut indices = (0..self.entries.len()).collect::<Vec<_>>();
        // Find the highest-scoring entries
        indices.sort_by(|&i, &j| self.entries[i].score().partial_cmp(&self.entries[j].score()).unwrap());
        let bound = 1 + rng.gen_range(0, indices.len());
        indices[indices.len() - 1 - rng.gen_range(0, bound)]
    }
    fn handle_approval(&mut self, param_idx: usize, did_approve: bool) {
        let mut rng = rand::thread_rng();
        let entry = &mut self.entries[param_idx];
        if did_approve {
            entry.upvotes += 1;
        } else {
            entry.downvotes += 1;
        }
        if rng.gen_ratio(1, 5) {
            let idx_1 = self.get_high_voted_idx(&mut rng);
            let idx_2 = self.get_high_voted_idx(&mut rng);
            let child = ParamPoolEntry {
                upvotes: 1,
                downvotes: 1,
                params: self.entries[idx_1].params.mutate(&self.entries[idx_2].params, &mut rng),
            };
            if self.entries.len() > 10 {
                self.entries.remove(self.get_low_voted_idx(&mut rng));
            }
            self.entries.push(child);
        }
        println!("Voted.\n{:#?}", self.entries);
    }
    fn gen(&self) -> (usize, expr::IExpr) {
        let mut rng = rand::thread_rng();
        let idx = self.get_high_voted_idx(&mut rng);
        let params = &self.entries[idx].params;
        (idx, params.gen_expr(&mut rng, 5))
    }
}

fn main() {
    let state = Mutex::new(ParamPool::new());
    rouille::start_server("localhost:8000", move |req| {
        router!(req,
            (GET) (/) => {
                let (i, expr) = state.lock().unwrap().gen();
                let serialized = rmp_serde::to_vec(&expr).unwrap();
                Response::redirect_303(format!("desc/{}/{}", i, &hex::encode(serialized)))
            },
            (GET) (/approve/{param_idx: usize}/{did_approve: bool}) => {
                let mut state = state.lock().unwrap();
                state.handle_approval(param_idx, did_approve);
                let (i, expr) = state.gen();
                let serialized = rmp_serde::to_vec(&expr).unwrap();
                Response::redirect_303(format!("/desc/{}/{}", i, &hex::encode(serialized)))
            },
            (GET) (/desc/{param_idx: usize}/{serialized_hex: String}) => {
                let serialized = try_or_400!(hex::decode(&serialized_hex));
                let expr: expr::IExpr = try_or_400!(rmp_serde::from_slice(&serialized));
                let html = std::fs::read_to_string("static/desc.html").unwrap();
                Response::html(html
                    .replace("%PARAM_IDX", &format!("{}", param_idx))
                    .replace("%FORMULA_HEX", &serialized_hex)
                    .replace("%FORMULA_SEXPR", &format!("{}", expr)))
            },
            (GET) (/img/{serialized_hex: String}) => {
                let serialized = try_or_400!(hex::decode(&serialized_hex));
                let expr: expr::IExpr = try_or_400!(rmp_serde::from_slice(&serialized));
                let mut png_data = Vec::new();
                expr.write_image_data(&mut png_data, 256, 256, 4);
                Response::from_data("image/png", png_data)
            },
            _ => {
                Response::text("404").with_status_code(404)
            }
        )
    });
}