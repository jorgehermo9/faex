use super::*;
use paste::paste;

#[test]
fn into_iter() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0b11110000u8;

    bitvec.push_bits(dummy_data, 8);

    let mut iter = bitvec.into_iter();
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter() {
    let mut bitvec = BitVec::new();
    let dummy_data = 0b11110000u8;

    bitvec.push_bits(dummy_data, 8);

    let mut iter = bitvec.iter();
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), None);
}

#[test]
fn for_loop() {
    let mut bitvec = BitVec::new();
    let dummy_data = u8::MAX;

    bitvec.push_bits(dummy_data, 8);

    for bit in bitvec {
        assert!(bit);
    }
}

#[test]
fn from_bool_slice() {
    let bools = [false, true, false, true, false, true, false, true];
    let bitvec = BitVec::from(&bools[..]);

    assert_eq!(bitvec.len(), 8);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert_eq!(bitvec.read_bits(0, 8), 0b10101010);
}

#[test]
fn from_empty_bool_slice() {
    let bools: [usize; 0] = [];
    let bitvec = BitVec::from(&bools[..]);

    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
fn from_bool_vec() {
    let bools = vec![false, true, false, true, false, true, false, true];
    let bitvec = BitVec::from(bools);

    assert_eq!(bitvec.len(), 8);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert_eq!(bitvec.read_bits(0, 8), 0b10101010);
}

#[test]
fn from_empty_bool_vec() {
    let bools: Vec<bool> = vec![];
    let bitvec = BitVec::from(bools);

    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
fn from_bool_array() {
    let bools = [false, true, false, true, false, true, false, true];
    let bitvec = BitVec::from(bools);

    assert_eq!(bitvec.len(), 8);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert_eq!(bitvec.read_bits(0, 8), 0b10101010);
}

#[test]
fn from_empty_bool_array() {
    let bools: [bool; 0] = [];
    let bitvec = BitVec::from(bools);

    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
fn from_bool_iterator() {
    let bools = [false, true, false, true, false, true, false, true];
    let bitvec = bools.into_iter().collect::<BitVec>();

    assert_eq!(bitvec.len(), 8);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert_eq!(bitvec.read_bits(0, 8), 0b10101010);
}

#[test]
fn from_empty_bool_iterator() {
    let bools: [bool; 0] = [];
    let bitvec = bools.into_iter().collect::<BitVec>();

    assert_eq!(bitvec.len(), 0);
    assert_eq!(bitvec.raw_data.len(), 0);
}

#[test]
fn extend_bool() {
    let mut bitvec = BitVec::new();
    let bools = [false, true, false, true, false, true, false, true];

    bitvec.extend(bools.iter().copied());

    assert_eq!(bitvec.len(), 8);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert_eq!(bitvec.read_bits(0, 8), 0b10101010);
}

#[test]
fn extend_bool_ref() {
    let mut bitvec = BitVec::new();
    let bools = [false, true, false, true, false, true, false, true];

    bitvec.extend(bools.iter());

    assert_eq!(bitvec.len(), 8);
    assert_eq!(bitvec.raw_data.len(), 1);

    assert_eq!(bitvec.read_bits(0, 8), 0b10101010);
}

macro_rules! test_for {
    ($($t:ty),*) => {
        $(
            paste!{
                #[test]
                fn [<from_ $t _slice>]() {
                    let values = [$t::MIN, $t::MAX];
                    let bitvec = BitVec::from(&values[..]);
                    let size = std::mem::size_of::<$t>() * 8;

                    assert_eq!(bitvec.len(), 2 * size);

                    assert_eq!(bitvec.read_bits(0,size), $t::MIN as usize);
                    assert_eq!(bitvec.read_bits(size,size), $t::MAX as usize);
                }

                #[test]
                fn [<from_empty_ $t _slice>]() {
                    let values: [$t; 0] = [];
                    let bitvec = BitVec::from(&values[..]);

                    assert!(bitvec.is_empty())
                }

                #[test]
                fn [<from_ $t _vec>]() {
                    let values = vec![$t::MIN, $t::MAX];
                    let bitvec = BitVec::from(values);
                    let size = std::mem::size_of::<$t>() * 8;

                    assert_eq!(bitvec.len(), 2 * size);

                    assert_eq!(bitvec.read_bits(0,size), $t::MIN as usize);
                    assert_eq!(bitvec.read_bits(size,size), $t::MAX as usize);
                }

                #[test]
                fn [<from_empty_ $t _vec>]() {
                    let values: Vec<$t> = vec![];
                    let bitvec = BitVec::from(values);

                    assert!(bitvec.is_empty())
                }

                #[test]
                fn [<from_ $t _array>]() {
                    let values = [$t::MIN, $t::MAX];
                    let bitvec = BitVec::from(values);
                    let size = std::mem::size_of::<$t>() * 8;

                    assert_eq!(bitvec.len(), 2 * size);

                    assert_eq!(bitvec.read_bits(0,size), $t::MIN as usize);
                    assert_eq!(bitvec.read_bits(size,size), $t::MAX as usize);
                }

                #[test]
                fn [<from_empty_ $t _array>]() {
                    let values: [$t; 0] = [];
                    let bitvec = BitVec::from(values);

                    assert!(bitvec.is_empty())
                }

                #[test]
                fn [<from_ $t _ref_array>]() {
                    let values = &[$t::MIN, $t::MAX];
                    let bitvec = BitVec::from(values);
                    let size = std::mem::size_of::<$t>() * 8;

                    assert_eq!(bitvec.len(), 2 * size);

                    assert_eq!(bitvec.read_bits(0,size), $t::MIN as usize);
                    assert_eq!(bitvec.read_bits(size,size), $t::MAX as usize);
                }

                #[test]
                fn [<from_empty_ $t _ref_array>]() {
                    let values: &[$t; 0] = &[];
                    let bitvec = BitVec::from(values);

                    assert!(bitvec.is_empty())
                }

                #[test]
                fn [<from_ $t _iterator>]() {
                    let values = [$t::MIN, $t::MAX];
                    let bitvec = values.into_iter().collect::<BitVec>();
                    let size = std::mem::size_of::<$t>() * 8;

                    assert_eq!(bitvec.len(), 2 * size);

                    assert_eq!(bitvec.read_bits(0,size), $t::MIN as usize);
                    assert_eq!(bitvec.read_bits(size,size), $t::MAX as usize);
                }

                #[test]
                fn [<from_empty_ $t _iterator>]() {
                    let values: [$t; 0] = [];
                    let bitvec = values.into_iter().collect::<BitVec>();

                    assert!(bitvec.is_empty())
                }

                #[test]
                fn [<extend_ $t>]() {
                    let mut bitvec = BitVec::new();
                    let values = [$t::MIN, $t::MAX];
                    let size = std::mem::size_of::<$t>() * 8;

                    bitvec.extend(values.iter().copied());

                    assert_eq!(bitvec.len(), 2 * size);
                    assert_eq!(bitvec.read_bits(0,size), $t::MIN as usize);
                    assert_eq!(bitvec.read_bits(size,size), $t::MAX as usize);
                }

                #[test]
                fn [<extend $t _ref>]() {
                    let mut bitvec = BitVec::new();
                    let values = [$t::MIN, $t::MAX];
                    let size = std::mem::size_of::<$t>() * 8;

                    bitvec.extend(values.iter());

                    assert_eq!(bitvec.len(), 2 * size);
                    assert_eq!(bitvec.read_bits(0,size), $t::MIN as usize);
                    assert_eq!(bitvec.read_bits(size,size), $t::MAX as usize);
                }
            }
        )*

    }
}
test_for!(u8, u16, usize);
