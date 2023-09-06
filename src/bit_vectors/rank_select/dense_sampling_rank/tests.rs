use crate::bit_vectors::{rank_select::tests_utils::test_rank_select_access_for, BitVec};

use super::*;
const WORD_SIZE: usize = std::mem::size_of::<usize>() * 8;
#[test]
fn new() {
    const TEST_DATA: [usize; 10] = [
        0b1000, 0b0010, 0b0000, 0b0110, 0b0000, 0b1010, 0b0000, 0b1011, 0b0100, 0b0001,
    ];
    let bv = BitVec::from(TEST_DATA);

    let rs = DenseSamplingRank::new(&bv, 4);

    // +1 because of the starting block (0) and +1 because of the last block as not all blocks are
    assert_eq!(rs.superblocks.len(), 4);
    assert_eq!(rs.superblock_size, 4 * WORD_SIZE);
    assert_eq!(rs.k, 4);

    assert_eq!(rs.superblocks(), &[0, 4, 9, 11]);
    let blocks = rs.blocks().iter().collect::<Vec<_>>();
    assert_eq!(&blocks, &[0, 1, 2, 2, 0, 0, 2, 2, 0, 1, 2]);
}

#[test]
fn new_when_fully_sampled() {
    const TEST_DATA: [usize; 8] = [
        0b1000, 0b0010, 0b0000, 0b0110, 0b0000, 0b1010, 0b0000, 0b1011,
    ];
    let bv = BitVec::from(TEST_DATA);
    let rs = DenseSamplingRank::new(&bv, 4);

    // +1 because of the starting block (0) and +1 because of the last block as not all blocks are
    // fully sampled
    assert_eq!(rs.superblocks.len(), 3);
    assert_eq!(rs.superblock_size, 4 * WORD_SIZE);
    assert_eq!(rs.k, 4);

    assert_eq!(rs.superblocks(), &[0, 4, 9]);
    let blocks = rs.blocks().iter().collect::<Vec<_>>();
    assert_eq!(&blocks, &[0, 1, 2, 2, 0, 0, 2, 2, 0]);
}

macro_rules! test_constant_time_for_k{
    ($( $k: expr ),*) => {
        $(
            paste::paste!{
            mod [<when_k_is_ $k>]{
                use super::*;

                #[test]
                fn new_when_len_is_multiple_of_superblock_size() {
                    const DESIRED_NUM_SUPERBLOCKS: usize = 6;
                    const BV_SIZE: usize = $k * DESIRED_NUM_SUPERBLOCKS * WORD_SIZE;

                    let bv = BitVec::from_value(true, BV_SIZE);

                    let rs = DenseSamplingRank::new(&bv, $k);

                    // + 1 block because of the starting block (0)
                    assert_eq!(rs.superblocks.len(), DESIRED_NUM_SUPERBLOCKS + 1);
                    assert_eq!(rs.superblock_size, $k * WORD_SIZE);
                    assert_eq!(rs.k, $k);

                    let mut acc = 0;
                    for superblock in rs.superblocks().iter().skip(1) {
                        acc += WORD_SIZE * $k;
                        assert_eq!(*superblock, acc);
                    }
                    assert_eq!(acc, BV_SIZE);
                }

                #[test]
                fn new_when_len_not_multiple_of_superblock_size() {
                    const DESIRED_NUM_SUPERBLOCKS: usize = 6;
                    const BV_SIZE: usize = $k * DESIRED_NUM_SUPERBLOCKS * WORD_SIZE + WORD_SIZE / 2;

                    let bv = BitVec::from_value(true, BV_SIZE);

                    let rs = DenseSamplingRank::new(&bv, $k);
                    // + 1 block because of the starting block (0)
                    assert_eq!(rs.superblocks.len(), DESIRED_NUM_SUPERBLOCKS + 1 + 1);
                    assert_eq!(rs.superblock_size, $k * WORD_SIZE);
                    assert_eq!(rs.k, $k);

                    let mut acc = 0;
                    for superblock in rs.superblocks().iter().skip(1).take(DESIRED_NUM_SUPERBLOCKS) {
                        acc += WORD_SIZE * $k;
                        assert_eq!(*superblock, acc);
                    }
                    // The last superblock is not fully sampled, as BV_SIZE is not multiple of the superblock size
                    assert_eq!(rs.superblocks().last().copied(), Some(BV_SIZE));
                    // Do not count the non-multiple block since it has no superblock associated
                    assert_eq!(acc, BV_SIZE - WORD_SIZE / 2);
                }
                #[test]
                fn heap_size_in_bits() {
                    let bv = BitVec::from_value(true, WORD_SIZE * 420 * $k);
                    let rs = DenseSamplingRank::new(&bv, $k);
                    let superblocks_overhead = (bv.len()/WORD_SIZE/$k + 1 )* WORD_SIZE;
                    let mut blocks_overhead = (WORD_SIZE * ($k-1)).bits_required() as usize * (bv.len() / WORD_SIZE + 1);

                    // CompactIntVec overhead may not be multiple of a word
                    let leftover_bits = blocks_overhead % WORD_SIZE;
                    if leftover_bits !=0 {
                        blocks_overhead += WORD_SIZE - leftover_bits;
                    }
                    assert_eq!(
                        rs.heap_size_in_bits(),
                        superblocks_overhead + blocks_overhead
                    );
                }

                test_rank_select_access_for!(DenseSamplingRank, $k);
            }
         }
        )*
    }
}

test_constant_time_for_k!(1, 2, 4, 5, 8, 16, 20, 32);
