use std::time::{Duration, Instant};
use std::{cmp::Ordering, hash::Hash};

use ordered_float::NotNan;
use std::collections::BinaryHeap;

use crate::cons::Cons;
use crate::map::Map;
use crate::prod::Prod;

static DIRS: [[i16; 2]; 4] = [[0, 1], [1, 0], [0, -1], [-1, 0]];

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Node {
    pub x: usize,
    pub y: usize,
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
    x: usize,
    y: usize,
    x_dest: usize,
    y_dest: usize,
    map: &'a Map,
    path: Vec<(usize, usize)>,
    progress: usize,
    distances: Vec<Vec<NotNan<f32>>>,
    distances_from_start: Vec<Vec<NotNan<f32>>>,
    distance_heuristics: Vec<Vec<NotNan<f32>>>,
    neighbour_distances: Vec<Vec<[NotNan<f32>; 4]>>,
    best_neighbours: Vec<Vec<usize>>,
    comm_out: Prod,
    comm_in: Cons,
}

impl<'a> MoonWalker<'_> {
    async fn new(map: &'_ Map) -> MoonWalker<'_> {
        let starter = map.dim / 2;

        let mut distances: Vec<Vec<NotNan<f32>>> = Vec::with_capacity(map.dim);
        let mut distances_from_start: Vec<Vec<NotNan<f32>>> = Vec::with_capacity(map.dim);
        let mut distance_heuristics: Vec<Vec<NotNan<f32>>> = Vec::with_capacity(map.dim);
        let mut neighbour_distances: Vec<Vec<[NotNan<f32>; 4]>> = Vec::with_capacity(map.dim);
        let mut best_neighbours: Vec<Vec<usize>> = Vec::with_capacity(map.dim);
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
            path: vec![(starter, starter)],
            progress: 0,
            distances: distances,
            distances_from_start,
            distance_heuristics: distance_heuristics,
            neighbour_distances: neighbour_distances,
            best_neighbours: best_neighbours,
            comm_out: Prod::new("moon-walker-out").await,
            comm_in: Cons::new("moon-walker-in").await,
        };

        for i in 0..map.dim {
            for j in 0..map.dim {
                if i > 0 {
                    res.neighbour_distances[i][j][0] = res.euc_dist(j, i, j, i - 1);
                }
                if j < res.map.dim - 1 {
                    res.neighbour_distances[i][j][1] = res.euc_dist(j, i, j + 1, i);
                }
                if i < res.map.dim - 1 {
                    res.neighbour_distances[i][j][2] = res.euc_dist(j, i, j, i + 1);
                }
                if j > 0 {
                    res.neighbour_distances[i][j][3] = res.euc_dist(j, i, j - 1, i);
                }
            }
        }

        res
    }

    fn euc_dist(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> NotNan<f32> {
        let x_dist = x1 as i16 - x2 as i16;
        let y_dist = y1 as i16 - y2 as i16;
        let z_dist = self.map.heights[y1][x1] as i16 - self.map.heights[y2][x2] as i16;

        NotNan::new(
            ((x_dist as i32 * x_dist as i32
                + y_dist as i32 * y_dist as i32
                + z_dist as i32 * z_dist as i32) as f32)
                .sqrt(),
        )
        .unwrap()
    }

    fn get_solution(&mut self, x_end: usize, y_end: usize) {
        self.path.clear();

        let mut prev = (x_end, y_end);
        loop {
            self.path.push(prev);
            let dir_ind: usize = self.best_neighbours[prev.1][prev.0] as usize;
            prev = (
                (prev.0 as i16 - DIRS[dir_ind][0]) as usize,
                (prev.1 as i16 - DIRS[dir_ind][1]) as usize,
            );
            if prev == (self.x, self.y) {
                break;
            }
        }
    }

    fn a_star(&mut self, x_end: usize, y_end: usize) {
        let mut q = BinaryHeap::new();
        q.push(Node {
            x: self.x,
            y: self.y,
            f: self.distances[self.y][self.x],
        });

        while !q.is_empty() {
            let curr = q.pop().unwrap();
            if curr.x == x_end && curr.y == y_end {
                self.get_solution(x_end, y_end);
                return;
            }

            for i in 0..4 {
                let [x_dir, y_dir] = DIRS[i];
                let x_neigh = x_dir + curr.x as i16;
                let y_neigh = y_dir + curr.y as i16;
                if x_neigh < 0
                    || x_neigh as usize == self.map.dim
                    || y_neigh < 0
                    || y_neigh as usize == self.map.dim
                {
                    continue;
                }

                let new_g = self.distances_from_start[curr.y][curr.x]
                    + self.neighbour_distances[curr.y][curr.x][i];
                if new_g < self.distances_from_start[y_neigh as usize][x_neigh as usize] {
                    self.distances_from_start[y_neigh as usize][x_neigh as usize] = new_g;
                    self.distance_heuristics[y_neigh as usize][x_neigh as usize] =
                        new_g + self.euc_dist(x_neigh as usize, y_neigh as usize, x_end, y_end);
                    self.best_neighbours[y_neigh as usize][x_neigh as usize] = i;
                    q.push(Node {
                        x: x_neigh as usize,
                        y: y_neigh as usize,
                        f: self.distance_heuristics[y_neigh as usize][x_neigh as usize],
                    });
                }
            }
        }
    }

    async fn change_destination(&mut self, x: usize, y: usize) {
        for i in 0..self.map.dim {
            for j in 0..self.map.dim {
                self.distances[i][j] = self.euc_dist(x, y, i, j);
                self.distances_from_start[i][j] = NotNan::new(f32::MAX).unwrap();
            }
        }

        self.x_dest = x;
        self.y_dest = y;
        self.distances_from_start[self.y][self.x] = NotNan::new(0.0).unwrap();
        self.a_star(x, y);
        self.progress = 0;
        self.send_path_report().await;
    }

    async fn send_pos_report(&mut self) {
        self.comm_out
            .send(&format!(
                "{},{},{},{},{}",
                self.x, self.y, self.x_dest, self.y_dest, self.map.dim
            ))
            .await;
        println!(
            "sent to the server: {},{},{},{},{}",
            self.x, self.y, self.x_dest, self.y_dest, self.map.dim
        );
    }

    async fn send_path_report(&mut self) {
        let message: String = self
            .path
            .iter()
            .rev()
            .map(|x| format!("({}, {}), ", x.0, x.1))
            .collect();
        self.comm_out.send(&message[..message.len() - 2]).await;
        println!("sent to the server: {}", &message[..message.len() - 2])
    }

    fn progress_further(&mut self) {
        if self.progress < self.path.len() - 1 {
            self.progress += 1;
            let new_pos = self.path[self.path.len() - 1 - self.progress];
            (self.x, self.y) = (new_pos.0, new_pos.1);
        }
    }

    pub async fn run_moon_walker() {
        let map = Map::new(256);
        let mut moon_walker = MoonWalker::new(&map).await;
        moon_walker.send_pos_report().await;
        let mut start = Instant::now();

        loop {
            let message;
            unsafe {
                message =
                    String::from_utf8_unchecked(moon_walker.comm_in.read_non_blocking().await);
            }
            if message.len() != 0 {
                let mut nums = message.split(|x| x == ' ');
                let (new_dest_x, new_dest_y) = (
                    nums.next().unwrap().parse::<usize>().unwrap(),
                    nums.next().unwrap().parse::<usize>().unwrap(),
                );
                if (moon_walker.x_dest, moon_walker.y_dest) != (new_dest_x, new_dest_y) {
                    println!("changed destination to {} {}", new_dest_x, new_dest_y);
                    moon_walker.change_destination(new_dest_x, new_dest_y).await;
                }
            }

            if Instant::now() - start > Duration::from_millis(500) {
                moon_walker.progress_further();
                moon_walker.send_pos_report().await;
                start = Instant::now();
            }
        }
    }
}
