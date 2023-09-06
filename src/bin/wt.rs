use faex::bit_vectors::rank_select::{DenseSamplingRank, Rank, SparseSamplingRank};
use faex::bit_vectors::{BitVec, RRRBitVec};
use faex::character_sequence::wavelet_tree::WaveletTree;
use faex::character_sequence::{CharacterAccess, CharacterRank};
use faex::profiling::HeapSize;
use faex::Build;
use rand::Rng;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::time::Instant;

fn main() {
    // let text_len = 100_000_000;

    // let mut rng = rand::thread_rng();

    // let text = (0..text_len)
    //     .map(|_| rng.gen::<u8>() as char)
    //     .collect::<String>();

    // read text from /home/jorge/tmp/english.200MB
    // read bytes and then convert with from_utf8_lossy
    let text = std::fs::read("/home/jorge/uni/tfg/experiments/datasets/english.200MB").unwrap();
    let text = String::from_utf8_lossy(&text).to_string();

    let start_construction = Instant::now();
    let spec = DenseSamplingRank::spec(4);
    // let spec = RRRVec::spec(63, 4);
    let wt = WaveletTree::spec(spec).build(&text);
    println!("sigma: {}", wt.alphabet().len());
    println!("size: {}", wt.len());
    println!("time construction: {:?}", start_construction.elapsed());
    // let text = text.chars().collect::<Vec<_>>();

    let start = Instant::now();
    let mut sum_ranks = 0;

    for i in 0..wt.len() {
        sum_ranks += wt.rank(wt.access(i).unwrap(), i).unwrap();
    }
    println!("sum_ranks: {}", sum_ranks);
    println!("time: {:?}", start.elapsed());
}
