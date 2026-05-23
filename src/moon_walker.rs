use std::{cmp::Ordering, hash::Hash};

use ordered_float::NotNan;
use std::collections::BinaryHeap;

use crate::map::Map;
use crate::moon_walker_out_comm::MoonWalkerOutCommunicator;

static DIRS: [[i16; 2]; 4] = [[0, 1], [1, 0], [0, -1], [-1, 0]];

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Node {
    pub x: i16,
    pub y: i16,
    pub f: NotNan<f32>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MoonWalker<'a> {
    x: i16,
    y: i16,
    x_dest: i16,
    y_dest: i16,
    map: &'a Map,
    path: Vec<(i16, i16)>,
    distances: Vec<Vec<NotNan<f32>>>,
    distances_from_start: Vec<Vec<NotNan<f32>>>,
    distance_heuristics: Vec<Vec<NotNan<f32>>>,
    neighbour_distances: Vec<Vec<[NotNan<f32>; 4]>>,
    best_neighbours: Vec<Vec<i8>>,
}

impl<'a> MoonWalker<'_> {
    fn new(map: &'_ Map) -> MoonWalker<'_> {
        let starter = map.dim as i16 / 2;

        let mut distances: Vec<Vec<NotNan<f32>>> = Vec::with_capacity(map.dim);
        let mut distances_from_start: Vec<Vec<NotNan<f32>>> = Vec::with_capacity(map.dim);
        let mut distance_heuristics: Vec<Vec<NotNan<f32>>> = Vec::with_capacity(map.dim);
        let mut neighbour_distances: Vec<Vec<[NotNan<f32>; 4]>> = Vec::with_capacity(map.dim);
        let mut best_neighbours: Vec<Vec<i8>> = Vec::with_capacity(map.dim);
        for _ in 0..map.dim {
            distances.push(vec![NotNan::new(0.0).unwrap(); map.dim]);
            distances_from_start.push(vec![NotNan::new(f32::MAX).unwrap(); map.dim]);
            distance_heuristics.push(vec![NotNan::new(f32::MAX).unwrap(); map.dim]);
            neighbour_distances.push(vec![[NotNan::new(0.0).unwrap(); 4]; map.dim]);
            best_neighbours.push(vec![0; map.dim]);
        }

        let mut res = MoonWalker {
            x: starter,
            y: starter,
            x_dest: starter,
            y_dest: starter,
            map: map,
            path: vec![],
            distances: distances,
            distances_from_start,
            distance_heuristics: distance_heuristics,
            neighbour_distances: neighbour_distances,
            best_neighbours,
        };

        for i in 0..map.dim {
            for j in 0..map.dim {
                if i > 0 {
                    res.neighbour_distances[i][j][0] =
                        res.euc_dist(j as i16, i as i16, j as i16, i as i16 - 1);
                }
                if j < res.map.dim - 1 {
                    res.neighbour_distances[i][j][1] =
                        res.euc_dist(j as i16, i as i16, j as i16 + 1, i as i16);
                }
                if i < res.map.dim - 1 {
                    res.neighbour_distances[i][j][2] =
                        res.euc_dist(j as i16, i as i16, j as i16, i as i16 + 1);
                }
                if j > 0 {
                    res.neighbour_distances[i][j][3] =
                        res.euc_dist(j as i16, i as i16, j as i16 - 1, i as i16);
                }
            }
        }

        res
    }

    fn euc_dist(&self, x1: i16, y1: i16, x2: i16, y2: i16) -> NotNan<f32> {
        let x_dist = x1 - x2;
        let y_dist = y1 - y2;
        let z_dist =
            self.map.heights[y1 as usize][x1 as usize] - self.map.heights[y2 as usize][x2 as usize];

        NotNan::new(
            ((x_dist as i32 * x_dist as i32
                + y_dist as i32 * y_dist as i32
                + z_dist as i32 * z_dist as i32) as f32)
                .sqrt(),
        )
        .unwrap()
    }

    fn get_solution(&mut self, x_end: i16, y_end: i16) {
        self.path.clear();

        let mut prev = (x_end, y_end);
        loop {
            self.path.push(prev);
            let dir_ind: usize = self.best_neighbours[prev.1 as usize][prev.0 as usize] as usize;
            prev = (prev.0 - DIRS[dir_ind][0], prev.1 - DIRS[dir_ind][1]);
            if prev == (self.x, self.y) {
                break;
            }
        }
    }

    fn a_star(&mut self, x_end: i16, y_end: i16) {
        let mut q = BinaryHeap::new();
        q.push(Node {
            x: self.x,
            y: self.y,
            f: self.distances[self.y as usize][self.x as usize],
        });

        while !q.is_empty() {
            let curr = q.pop().unwrap();
            if curr.x == x_end && curr.y == y_end {
                self.get_solution(x_end, y_end);
                return;
            }

            for i in 0..4 {
                let [x_dir, y_dir] = DIRS[i];
                let x_neigh = x_dir + curr.x;
                let y_neigh = y_dir + curr.y;
                if x_neigh < 0
                    || x_neigh == self.map.dim as i16
                    || y_neigh < 0
                    || y_neigh == self.map.dim as i16
                {
                    continue;
                }

                let new_g = self.distances_from_start[curr.y as usize][curr.x as usize]
                    + self.neighbour_distances[curr.y as usize][curr.x as usize][i];
                if new_g < self.distances_from_start[y_neigh as usize][x_neigh as usize] {
                    self.distances_from_start[y_neigh as usize][x_neigh as usize] = new_g;
                    self.distance_heuristics[y_neigh as usize][x_neigh as usize] =
                        new_g + self.euc_dist(x_neigh, y_neigh, x_end, y_end);
                    self.best_neighbours[y_neigh as usize][x_neigh as usize] = i as i8;
                    q.push(Node {
                        x: x_neigh,
                        y: y_neigh,
                        f: self.distance_heuristics[y_neigh as usize][x_neigh as usize],
                    });
                }
            }
        }
    }

    pub fn change_destination(&mut self, x: i16, y: i16) {
        for i in 0..self.map.dim {
            for j in 0..self.map.dim {
                self.distances[i][j] = self.euc_dist(x, y, i as i16, j as i16);
                self.distances_from_start[i][j] = NotNan::new(f32::MAX).unwrap();
            }
        }

        self.x_dest = x;
        self.y_dest = y;
        self.distances_from_start[self.y as usize][self.x as usize] = NotNan::new(0.0).unwrap();
        self.a_star(x, y);
    }

    pub async fn run_moon_walker() -> Result<(), ()> {
        let map = Map::new(256);
        let mut moon_walker = MoonWalker::new(&map);
        let moon_walker_out_communicator = MoonWalkerOutCommunicator::new().await;
        loop {
            let (new_x, new_y) = (50i16, 100i16);
            if (moon_walker.x, moon_walker.y) != (new_x, new_y) {
                moon_walker.change_destination(50, 100);
                let msg: String = moon_walker
                    .path
                    .iter()
                    .map(|x| format!("({}, {}), ", x.0, x.1))
                    .collect();
                moon_walker_out_communicator.send(msg.as_str()).await;
            }
        }
    }
}
