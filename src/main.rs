use jandom::Random;
use rayon::prelude::*; // Import Rayon traits for parallel iterators
use std::env;
use std::time::Instant;

const X1_MULTIPLIER: i32 = 0x4c1906;
const X2_MULTIPLIER: i32 = 0x5ac0db;
const Z1_MULTIPLIER: i64 = 0x4307a7;
const Z2_MULTIPLIER: i32 = 0x5f24f;
const SEED_XOR: i64 = 0x3ad8025f;

fn calculate_seed(world_seed: i64, x: i32, z: i32) -> i64 {
    let x1 = x.wrapping_mul(x).wrapping_mul(X1_MULTIPLIER) as i64;
    let x2 = x.wrapping_mul(X2_MULTIPLIER) as i64;
    let z1 = (z.wrapping_mul(z) as i64).wrapping_mul(Z1_MULTIPLIER);
    let z2 = z.wrapping_mul(Z2_MULTIPLIER) as i64;

    world_seed
        .wrapping_add(x1)
        .wrapping_add(x2)
        .wrapping_add(z1)
        .wrapping_add(z2)
        ^ SEED_XOR
}

fn is_slime_chunk(seed: i64) -> bool {
    let mut rng = Random::new(seed);
    rng.next_i32_bounded(10) == 0
}

fn get_slime_neighbor_count(world_seed: i64, x: i32, z: i32) -> i32 {
    let mut count = 0;

    for dx in -1..=1 {
        for dz in -1..=1 {
            let neighbor_x = x + dx;
            let neighbor_z = z + dz;

            let neighbor_seed = calculate_seed(world_seed, neighbor_x, neighbor_z);
            if is_slime_chunk(neighbor_seed) {
                count += 1;
            }
        }
    }

    count
}

fn parse_argument<T: std::str::FromStr>(arg: Option<String>, name: &str) -> T {
    arg.and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        panic!("{} not provided or invalid", name);
    })
}

fn squared_distance(x: i32, z: i32) -> i32 {
    x.pow(2) + z.pow(2)
}

fn main() {
    let start_time = Instant::now();

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("Usage: <program> <seed> <radius>");
    }

    let world_seed: i64 = parse_argument(args.get(1).cloned(), "Seed");
    let radius: i32 = parse_argument(args.get(2).cloned(), "Radius");

    let coordinates: Vec<(i32, i32)> = (-radius..=radius)
        .flat_map(|x| (-radius..=radius).map(move |z| (x, z)))
        .collect();

    let (best_chunk, max_slime_neighbors) = coordinates
        .par_iter()
        .map(|&(x, z)| {
            let seed = calculate_seed(world_seed, x, z);
            if !is_slime_chunk(seed) {
                let slime_neighbors = get_slime_neighbor_count(world_seed, x, z);
                ((x, z), slime_neighbors)
            } else {
                ((x, z), -1)
            }
        })
        .reduce(
            || ((0, 0), -1),
            |((best_x, best_z), best_count), ((x, z), count)| {
                if count > best_count {
                    ((x, z), count)
                } else if count == best_count {
                    let best_distance = squared_distance(best_x, best_z);
                    let new_distance = squared_distance(x, z);
                    if new_distance < best_distance {
                        ((x, z), count)
                    } else {
                        ((best_x, best_z), best_count)
                    }
                } else {
                    ((best_x, best_z), best_count)
                }
            },
        );

    let duration = start_time.elapsed();

    println!(
        "Took {:?} to find ({}, {}) with {} neighbours",
        duration, best_chunk.0, best_chunk.1, max_slime_neighbors
    );
    println!(
        "https://www.chunkbase.com/apps/slime-finder#seed={}&platform=java&x={}&z={}",
        world_seed,
        best_chunk.0 * 16,
        best_chunk.1 * 16
    )
}
