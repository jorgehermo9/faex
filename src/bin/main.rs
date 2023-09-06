use faex::bit_vectors::rank_select::{DenseSamplingRank, Rank, SparseSamplingRank};
use faex::bit_vectors::{BitVec, RRRBitVec};
use faex::profiling::HeapSize;
use faex::Build;
use rand::seq::SliceRandom;
use std::time::Instant;

fn main() {
    let iters = 100_000_000;
    let size = iters;

    let bv = BitVec::from_value(true, size);

    // let bv = (0..size).map(|i| (i % 62) == 0).collect::<BitVec>();
    dbg!(bv.rank(bv.len()));
    println!("Original bv size in KiB: {}", bv.heap_size_in_kib());

    let plain_size = bv.heap_size_in_kib();
    let start_new = Instant::now();
    // let bv = RRRVec::spec(63, 32).build(bv);
    dbg!(start_new.elapsed());
    dbg!("RRRVec constructed");
    let rrr_size = bv.heap_size_in_kib();
    println!("RRRVec size in Kib: {}", rrr_size);
    println!(
        "Compressed: {}% of original",
        rrr_size as f64 / plain_size as f64 * 100.0
    );

    let rs = DenseSamplingRank::spec(64).build(bv.clone());
    dbg!(rs.heap_size_in_bits());
    let mut sum = 0;

    let mut random_rank_positions = (0..=bv.len()).collect::<Vec<usize>>();
    random_rank_positions.shuffle(&mut rand::thread_rng());
    dbg!("Starting rank");
    let start = Instant::now();

    for i in random_rank_positions {
        std::hint::black_box(rs.rank(i));
    }
    let elapsed_rs = start.elapsed();

    println!("Elapsed faex time: {}, sum: {sum}", elapsed_rs.as_millis());

    let rank_size = rs.rank_support().heap_size_in_kib();
    dbg!(rank_size, rrr_size);
    let overhead = (rank_size as f64 / rrr_size as f64) * 100.0;
    println!("Overhead: {}%", overhead);
}
