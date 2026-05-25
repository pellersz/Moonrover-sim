use rand::{RngExt, rngs::ThreadRng};

pub struct Map {
    pub dim: usize,
    pub heights: Vec<Vec<u16>>,
}

impl Map {
    pub fn new(dim: usize) -> Map {
        let mut res: Map = Map {
            dim: dim,
            heights: Vec::with_capacity(dim),
        };

        let mut randgen: ThreadRng = rand::rng();

        for i in 0..dim {
            res.heights.push(Vec::with_capacity(dim));
            for _ in 0..dim {
                res.heights[i].push(randgen.random_range(0..10));
            }
        }

        res
    }
}
