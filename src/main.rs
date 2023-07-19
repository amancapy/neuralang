use rand::{distributions::Uniform, prelude::*};
use rayon::{prelude::*, iter::plumbing};
use std::{
    thread,
    borrow::BorrowMut,
    sync::{Arc, Mutex, RwLock},
    time::*, collections::HashSet,
};


const W_SIZE: usize = 1000;
const N_CELLS: usize = 25;
const CHUNK_SIZE: usize = W_SIZE / N_CELLS;


fn normalize_2d((i, j): (f64, f64)) -> (f64, f64) {
    let norm = (i.powi(2) + j.powi(2)).sqrt();

    (i / norm, j / norm)
}

fn add_2d((i, j): (f64, f64), (k, l): (f64, f64)) -> (f64, f64) {
    (i + k, j + l)
}

fn scale_2d((i, j): (f64, f64), c: f64) -> (f64, f64) {
    (i * c, j * c)
}

fn dist_2d((i1, j1): (f64, f64), (i2, j2): (f64, f64)) -> f64 {
    ((i1 - i2).powi(2) + (j1 - j2).powi(2)).sqrt()
}

fn one_to_two(ij: usize) -> (usize, usize) {
    ((ij - ij % W_SIZE) / W_SIZE, ij % W_SIZE)
}

fn two_to_one((i, j): (usize, usize)) -> usize {
    i * W_SIZE + j
}

fn dir_from_theta(theta: f64) -> (f64, f64) {
    (theta.cos(), theta.sin())
}

fn equal_idx((a, b): (usize, usize), (c, d): (usize, usize)) -> bool {
    a == c && b == d
}

pub fn pos_to_chunk(pos: (f64, f64)) -> (usize, usize) {
    let c = CHUNK_SIZE as f64;
    let i = ((pos.0 - (pos.0 % c)) / c) as usize;
    let j = ((pos.1 - (pos.1 % c)) / c) as usize;

    (i, j)
}

pub fn balls_collide (b1: &Ball, b2: &Ball) -> bool {
    let rad_dist = dist_2d(b1.pos, b2.pos);
    let (r1, r2) = (b1.radius, b2.radius);

    rad_dist < r1 + r2

}


pub fn resolve_balls (b1: &mut Ball, b2: &mut Ball) {
    let (i1, j1) = b1.pos;
    let (i2, j2) = b2.pos;
    let c1c2 = (i2 - i1, j2 - j1);
    let half_dist = scale_2d(c1c2, 0.5);

    b1.pos = add_2d((i1, j1), scale_2d(half_dist, -1.));
    b2.pos = add_2d((i2, j2), half_dist);
}


pub struct Ball {
    radius: f64,
    pos: (f64, f64),
    rotation: f64,
    speed: f64,
    chunk: (usize, usize),
    id: usize
}


struct Chunk {
    balls: Vec<Ball>,
    cells: Vec<Vec<HashSet<usize>>>,
    cell_size: usize,
    ball_id: usize
}

impl Chunk {
    pub fn new(n_cells: usize) -> Self {
        assert!(W_SIZE % n_cells == 0);
        Chunk { balls: 
            vec![],
            cells: (0..n_cells).into_iter().map(|_| {
                (0..n_cells).into_iter().map(|_| {
                    HashSet::new()
                }).collect()
            }).collect(),
            cell_size: W_SIZE / n_cells,
            ball_id: 0
         }
    }

    pub fn add_ball(&mut self, radius: f64, pos: (f64, f64), rotation: f64, speed: f64) {
        let (i, j) = pos_to_chunk(pos);
        self.balls.push(Ball {radius: radius, pos: pos, rotation: rotation, speed: speed, chunk: (i, j), id: self.ball_id});
        self.cells[i][j].insert(self.ball_id);
        self.ball_id += 1;
    }

    

    pub fn move_balls(&mut self) {
        let w = W_SIZE as f64;
        self.balls.iter_mut().for_each(|ball| {
            let move_vec = scale_2d(dir_from_theta(ball.rotation), ball.speed);
            let (ni, nj) = add_2d(ball.pos, move_vec);

            if !(ni < 1. || nj < 1. || ni > w || nj > w) {
                ball.pos = add_2d(ball.pos, move_vec);

            }

            let (ni, nj) = pos_to_chunk(ball.pos);
            if !equal_idx((ni, nj), ball.chunk) {
                let (oi, oj) = ball.chunk;
                ball.chunk = (ni, nj);

                self.cells[oi][oj].remove(&ball.id);
                self.cells[ni][nj].insert(ball.id);
            }
        })
    }

    pub fn check_collisions(&mut self) {
        for i in 1..N_CELLS - 1 {
            for j in 1..N_CELLS - 1 {
                for id1 in &self.cells[i][j] {
                    let mut b1 = &self.balls[*id1];

                    for (di, dj) in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 0), (0, 1), (1, -1), (1, 0), (1, 1)] {
                        let (ni, nj) = ((i as isize + di) as usize, (j as isize + dj) as usize);
                        for id2 in &self.cells[ni][nj] {
                            if *id1 != *id2 {
                                let mut b2 = &self.balls[*id2];
                                if balls_collide(b1, b2) {
                                    // resolve_balls(b1, b2);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.move_balls();
        self.check_collisions();
    }
}


fn main() {
    assert!(W_SIZE % N_CELLS == 0); 
    let mut chunk = Chunk::new(N_CELLS);
    let rdist = Uniform::new(0., W_SIZE as f64 - 10.);
    let mut rng = thread_rng();

    for i in 1..10000{chunk.add_ball(5., (rng.sample(rdist), rng.sample(rdist)), 0.7853981633974483, 5.);}


    for i in 1..10000000_u64 {
        if i % 1 == 0 {println!("{}", i)}
        chunk.step();
    }

    println!("{:?}, {:?}", chunk.balls[0].pos, chunk.balls[0].chunk);
}