use crate::bit_vectors::rank_select::tests_utils::test_rank_select_access_for;

use super::*;

mod iter;

#[test]
fn new() {
    let bitvec = BitVec::new();
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
fn with_capacity() {
    let capacity = BitVec::CONTAINER_WIDTH * 4;
    let bitvec = BitVec::with_capacity(capacity);
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.capacity(), capacity);
    assert_eq!(
        bitvec.raw_data.capacity(),
        capacity / BitVec::CONTAINER_WIDTH
    );
}

#[test]
fn with_capacity_when_capacity_is_not_multiple() {
    let capacity = 8;
    let bitvec = BitVec::with_capacity(capacity);
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
    // capacity is rounded up to the nearest multiple of CONTAINER_WIDTH
    assert!(bitvec.capacity() >= capacity);
    assert!(bitvec.raw_data.capacity() >= 1);
}

#[test]
fn push() {
    let mut bitvec = BitVec::new();
    bitvec.push(true);
    bitvec.push(false);

    assert_eq!(bitvec.len(), 2);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert!(bitvec.read(0));
    assert!(!bitvec.read(1));
}

#[test]
fn push_bits() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0b10101010u8;
    for _ in 0..8 {
        bitvec.push_bits(dummy_data, 8);
    }

    bitvec.push_bits(usize::MAX, 64);

    assert_eq!(bitvec.len(), 8 * 8 + 64);
    assert_eq!(bitvec.raw_data.len(), 2);
}

#[test]
fn push_bits_when_value_spans_two_words() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0b10101010u8;
    bitvec.push_bits(dummy_data, 8);

    bitvec.push_bits(usize::MAX, 64);

    assert_eq!(bitvec.len(), 8 + 64);
    assert_eq!(bitvec.raw_data.len(), 2);

    assert_eq!(bitvec.read_bits(0, 8), dummy_data as usize);
    assert_eq!(bitvec.read_bits(8, 64), usize::MAX);
}

#[test]
fn push_bits_with_no_width() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(0u8, 0);
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
fn push_bits_when_width_overflows_value() {
    // in this case, the value is padded with 0s
    let mut bitvec = BitVec::new();
    bitvec.push_bits(u8::MAX, 9);
}

#[test]
fn push_bits_when_value_does_not_fit_in_width() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(0b10101010u8, 4);
    assert_eq!(bitvec.len(), 4);
    assert_eq!(bitvec.read_bits(0, 4), 0b1010);
}

#[test]
#[should_panic]
fn push_bits_fails_when_width_overflows_container() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(usize::MAX, 65);
}

#[test]
fn pop() {
    let mut bitvec = BitVec::new();
    bitvec.push(true);
    bitvec.push(false);

    assert_eq!(bitvec.len(), 2);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert!(!bitvec.pop());
    assert_eq!(bitvec.len(), 1);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert!(bitvec.pop());
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
#[should_panic]
fn pop_fails_when_no_bits() {
    let mut bitvec = BitVec::new();
    bitvec.pop();
}

#[test]
fn pop_bits() {
    let mut bitvec = BitVec::new();
    bitvec.push(true);
    bitvec.push(false);

    assert_eq!(bitvec.len(), 2);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert_eq!(bitvec.pop_bits(2), 0b01);
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
#[should_panic]
fn pop_bits_fails_when_not_enough_bits() {
    let mut bitvec = BitVec::new();
    bitvec.push(true);
    bitvec.pop_bits(3);
}

#[test]
fn read() {
    let mut bitvec = BitVec::new();
    bitvec.push(true);
    bitvec.push(false);

    assert!(bitvec.read(0));
    assert!(!bitvec.read(1));
}

#[test]
#[should_panic]
fn read_fails_when_index_out_of_bounds() {
    let mut bitvec = BitVec::new();
    bitvec.push(true);
    bitvec.read(1);
}

#[test]
fn read_bits() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0b11110000u8;

    bitvec.push_bits(dummy_data, 8);
    bitvec.push_bits(usize::MAX, 64);

    assert_eq!(bitvec.read_bits(0, 4), 0b0000);
    assert_eq!(bitvec.read_bits(4, 4), 0b1111);
    assert_eq!(bitvec.read_bits(0, 8), dummy_data as usize);
    assert_eq!(bitvec.read_bits(8, bitvec.len() - 8), usize::MAX);
}

#[test]
fn read_bits_when_range_span_two_words() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0usize;

    bitvec.push_bits(dummy_data, 64);
    bitvec.push_bits(usize::MAX, 64);

    assert_eq!(bitvec.read_bits(60, 8), 0b11110000);
}

#[test]
fn read_bits_when_no_width() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(0u8, 0);
    assert_eq!(bitvec.read_bits(0, 0), 0);
}

#[test]
#[should_panic]
fn read_bits_fails_when_width_overflows_container() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(usize::MAX, 64);
    bitvec.read_bits(0, 64 + 1);
}

#[test]
#[should_panic]
fn read_bits_fails_when_index_out_of_bounds() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(usize::MAX, 64);
    bitvec.read_bits(64, 2);
}

#[test]
fn set() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0u8;

    bitvec.push_bits(dummy_data, 8);

    bitvec.set(4, true);

    assert!(bitvec.read(4));
}

#[test]
#[should_panic]
fn set_fails_when_index_out_of_bounds() {
    let mut bitvec = BitVec::new();
    bitvec.push(true);
    bitvec.set(1, true);
}

#[test]
fn set_bits() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0u8;

    bitvec.push_bits(dummy_data, 8);

    bitvec.set_bits(4..8, 0b1111u8);

    assert_eq!(bitvec.read_bits(0, 8), 0b11110000u8 as usize);
}

#[test]
fn set_bits_when_range_spans_two_words() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0usize;

    bitvec.push_bits(dummy_data, 64);
    bitvec.push_bits(dummy_data, 64);

    bitvec.set_bits(60..68, 0b11111111u8);

    assert_eq!(bitvec.read_bits(60, 8), 0b11111111u8 as usize);
    assert_eq!(bitvec.read_bits(0, 60), 0usize);
    assert_eq!(bitvec.read_bits(68, 60), 0usize);
}

#[test]
fn set_bits_when_no_width() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(0u8, 0);
    bitvec.set_bits(0..0, 0u8);
}

#[test]
fn set_bits_fails_when_value_does_not_fit_in_width() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(0u8, 8);
    bitvec.set_bits(0..4, 0b10101010u8);
    assert_eq!(bitvec.read_bits(0, 8), 0b00001010u8 as usize);
}

#[test]
#[should_panic]
fn set_bits_fails_when_index_out_of_bounds() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(0u8, 8);
    bitvec.set_bits(7..9, 0u8);
}

#[test]
#[should_panic]
fn set_bits_fails_when_width_overflows_container() {
    let mut bitvec = BitVec::new();
    bitvec.push_bits(usize::MAX, 64);
    bitvec.set_bits(0..65, 0u8);
}

#[test]
fn default() {
    let bitvec = BitVec::default();
    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
fn is_empty() {
    let mut bitvec = BitVec::new();
    assert!(bitvec.is_empty());

    bitvec.push(true);
    assert!(!bitvec.is_empty());
}

#[test]
fn raw_data() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0b10101010u8;

    bitvec.push_bits(dummy_data, 8);

    let raw_data = bitvec.raw_data();
    assert_eq!(raw_data.len(), 1);
    assert_eq!(raw_data[0], dummy_data as usize);
}

#[test]
fn heap_size_bits_bits() {
    let mut bitvec = BitVec::new();
    let width = std::mem::size_of::<usize>() * 8;
    let dummy_data = usize::MAX;

    bitvec.push_bits(dummy_data, width);
    bitvec.push_bits(dummy_data, width);

    assert_eq!(bitvec.heap_size_in_bits(), width * 2);
}

#[test]
fn access() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0b10101010u8;

    bitvec.push_bits(dummy_data, 8);

    assert!(!bitvec.access(0).unwrap());
    assert!(bitvec.access(1).unwrap());
    assert!(!bitvec.access(2).unwrap());
    assert!(bitvec.access(3).unwrap());
    assert!(!bitvec.access(4).unwrap());
    assert!(bitvec.access(5).unwrap());
    assert!(!bitvec.access(6).unwrap());
    assert!(bitvec.access(7).unwrap());
    assert!(bitvec.access(8).is_none());
}

test_rank_select_access_for!(BitVec,);
