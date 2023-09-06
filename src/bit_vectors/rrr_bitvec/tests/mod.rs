use super::*;
use crate::bit_vectors::rank_select::tests_utils::test_rank_select_access_for;

#[test]
fn encode() {
    let block = 0b0010;
    let (class, offset) = RRRBitVec::encode(block, 4);
    assert_eq!(1, class);
    assert_eq!(2, offset);

    let block = 0b1101;
    let (class, offset) = RRRBitVec::encode(block, 4);
    assert_eq!(3, class);
    assert_eq!(1, offset);
}

#[test]
fn decode() {
    let (class, offset) = (1, 2);
    let block = RRRBitVec::decode(class, offset, 4, 4);
    assert_eq!(0b0010, block);

    let (class, offset) = (3, 1);
    let block = RRRBitVec::decode(class, offset, 4, 4);
    assert_eq!(0b1101, block);
}

#[test]
fn decode_is_the_inverse_of_encode() {
    let mut block = 0;
    for b in 1..=64 {
        block |= (1 << (b - 1)) * (b % 2);
        let (class, offset) = RRRBitVec::encode(block, b);
        let decoded_block = RRRBitVec::decode(class, offset, b, b);
        assert_eq!(block, decoded_block);
    }
}

#[test]
fn new() {
    let b = 4;
    let data: [u8; 5] = [0b01000001, 0b01100000, 0b01010000, 0b11010000, 0b10000001];
    let bv = BitVec::from(data);
    let rrr = RRRBitVec::new(bv, b, 4);

    let expected_classes = [1usize, 1, 0, 2, 0, 2, 0, 3, 1, 1];
    let classes = rrr.classes().iter().collect::<Vec<_>>();
    assert_eq!(&expected_classes, classes.as_slice());

    let expected_lengths = [0usize, 2, 3, 2, 0];
    let lengths = rrr.lengths().to_vec();
    assert_eq!(&expected_lengths, lengths.as_slice());

    let offsets = rrr.offsets();
    assert_eq!(offsets.read_bits(0, 2), 0b11);
    assert_eq!(offsets.read_bits(2, 2), 0b01);
    assert_eq!(offsets.read_bits(4, 0), 0b0);
    assert_eq!(offsets.read_bits(4, 3), 0b010);
    assert_eq!(offsets.read_bits(7, 0), 0b0);
    assert_eq!(offsets.read_bits(7, 3), 0b100);
    assert_eq!(offsets.read_bits(10, 0), 0b0);
    assert_eq!(offsets.read_bits(10, 2), 0b01);
    assert_eq!(offsets.read_bits(12, 2), 0b11);
    assert_eq!(offsets.read_bits(14, 2), 0b00);

    let expected_offset_samples = [0, 7, 12];
    let offset_samples = rrr.offset_samples().iter().collect::<Vec<_>>();
    assert_eq!(&expected_offset_samples, offset_samples.as_slice());

    // +1 because of the starting block (0) and +1 because of the last block as not all blocks are
    // fully sampled
    let expected_rank_samples = [0usize, 4, 9, 11];
    let rank_samples = rrr.rank_samples().iter().collect::<Vec<_>>();
    assert_eq!(&expected_rank_samples, rank_samples.as_slice());

    assert_eq!(rrr.total_rank(), 11);
    assert_eq!(rrr.len(), 40);
    assert_eq!(rrr.b(), 4);
    assert_eq!(rrr.k(), 4);
}

#[test]
fn new_when_fully_sampled() {
    let b = 4;
    let data: [u8; 6] = [
        0b01000001, 0b01100000, 0b01010000, 0b11010000, 0b10000001, 0b10000001,
    ];
    let bv = BitVec::from(data);
    let rrr = RRRBitVec::new(bv, b, 4);

    let expected_classes = [1usize, 1, 0, 2, 0, 2, 0, 3, 1, 1, 1, 1];
    let classes = rrr.classes().iter().collect::<Vec<_>>();
    assert_eq!(&expected_classes, classes.as_slice());

    let expected_lengths = [0usize, 2, 3, 2, 0];
    let lengths = rrr.lengths().to_vec();
    assert_eq!(&expected_lengths, lengths.as_slice());

    let offsets = rrr.offsets();
    assert_eq!(offsets.read_bits(0, 2), 0b11);
    assert_eq!(offsets.read_bits(2, 2), 0b01);
    assert_eq!(offsets.read_bits(4, 0), 0b0);
    assert_eq!(offsets.read_bits(4, 3), 0b010);
    assert_eq!(offsets.read_bits(7, 0), 0b0);
    assert_eq!(offsets.read_bits(7, 3), 0b100);
    assert_eq!(offsets.read_bits(10, 0), 0b0);
    assert_eq!(offsets.read_bits(10, 2), 0b01);
    assert_eq!(offsets.read_bits(12, 2), 0b11);
    assert_eq!(offsets.read_bits(14, 2), 0b00);
    assert_eq!(offsets.read_bits(16, 2), 0b11);
    assert_eq!(offsets.read_bits(18, 2), 0b00);

    let expected_offset_samples = [0, 7, 12, 20];
    let offset_samples = rrr.offset_samples().iter().collect::<Vec<_>>();
    assert_eq!(&expected_offset_samples, offset_samples.as_slice());

    // +1 because of the starting block (0) and +1 because of the last block as not all blocks are
    // fully sampled
    let expected_rank_samples = [0usize, 4, 9, 13];
    let rank_samples = rrr.rank_samples().iter().collect::<Vec<_>>();
    assert_eq!(&expected_rank_samples, rank_samples.as_slice());

    assert_eq!(rrr.total_rank(), 13);
    assert_eq!(rrr.len(), 48);
    assert_eq!(rrr.b(), 4);
    assert_eq!(rrr.k(), 4);
}

macro_rules! test_for_k{
    ($($k:expr),*) =>{
        $(
            paste::paste! {
                mod [<when_k_is_ $k>]{
                    use super::*;
                    const K: usize = $k;

                    test_rank_select_access_for!(RRRBitVec, B, K);
                }
            }
        )*
    }
}
macro_rules! test_for_b {

    ($($b:expr),*) =>{
        $(
            paste::paste! {
                mod [<when_b_is $b>]{
                    use super::*;
                    const B: usize = $b;
                    test_for_k!(4, 16, 64, 128, 256);
                }
            }
        )*
    }
}

test_for_b!(1, 2, 3, 4, 15, 16, 20, 31, 32, 63, 64);
