use super::*;
use paste::paste;

// TODO: Macro with tests so that we can test for different int_widths as we
// do with k in sparse and constant time rank

#[test]
#[should_panic]
fn new_fails_when_int_width_is_greater_than_word_size() {
    let invalid_width = BitVec::CONTAINER_WIDTH + 1;
    CompactIntVec::new(invalid_width);
}

macro_rules! test_for_width{
    ($( $width: expr ),*) => {
        $(
            paste!{
                mod [<when_width_is_ $width>]{
                    use super::*;

                    const WIDTH : usize = $width;
                    const MIN_VALUE: usize = 0;
                    // Do not use pow, since it would overflow for 64 bits.
                    const MAX_VALUE: usize = match usize::MAX.checked_shr(usize::BITS as u32 - WIDTH as u32){
                        Some(v) => v,
                        None => 0,
                    };
                    const MID_VALUE: usize = MAX_VALUE / 2;



                    #[test]
                    fn new() {
                        let compact_int_vec = CompactIntVec::new(WIDTH);
                        assert_eq!(compact_int_vec.len(), 0);
                        assert_eq!(compact_int_vec.raw_data.len(), 0);
                    }


                    #[test]
                    fn with_capacity() {
                        let capacity = 16;
                        let compact_int_vec = CompactIntVec::with_capacity(WIDTH, capacity);
                        assert_eq!(compact_int_vec.len(), 0);
                        assert!(compact_int_vec.capacity() >= capacity);
                    }

                    #[test]
                    fn width() {
                        let compact_int_vec = CompactIntVec::new(WIDTH);
                        assert_eq!(compact_int_vec.width(), WIDTH);
                    }

                    #[test]
                    fn push() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);

                        let range = 0..WIDTH;

                        assert_eq!(compact_int_vec.len(), 0);
                        for i in range.clone() {
                            compact_int_vec.push(2_usize.pow(i as u32));
                        }
                        assert_eq!(compact_int_vec.len(), WIDTH);

                        for i in range {
                            assert_eq!(compact_int_vec.get(i).unwrap(), 2_usize.pow(i as u32));
                        }
                    }

                    #[test]
                    #[should_panic]
                    fn push_fails_when_value_does_not_fit() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        compact_int_vec.push(MAX_VALUE + 1);
                    }

                    #[test]
                    fn pop() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        compact_int_vec.push(MIN_VALUE);
                        compact_int_vec.push(MAX_VALUE);

                        assert_eq!(compact_int_vec.pop().unwrap(), MAX_VALUE);
                        assert_eq!(compact_int_vec.pop().unwrap(), MIN_VALUE);
                    }

                    #[test]
                    fn pop_when_empty() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        assert_eq!(compact_int_vec.pop(), None);
                    }

                    #[test]
                    fn get() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        compact_int_vec.push(MIN_VALUE);
                        compact_int_vec.push(MAX_VALUE);

                        assert_eq!(compact_int_vec.get(0).unwrap(), MIN_VALUE);
                        assert_eq!(compact_int_vec.get(1).unwrap(), MAX_VALUE);
                    }

                    #[test]
                    fn get_when_index_out_of_bounds() {
                        if(WIDTH==0){
                            return;
                        }
                        let compact_int_vec = CompactIntVec::new(WIDTH);
                        assert_eq!(compact_int_vec.get(0),None);
                        assert_eq!(compact_int_vec.get(1),None);
                    }

                    #[test]
                    fn set() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        let range = 0..WIDTH;

                        for _ in range.clone() {
                            compact_int_vec.push(0usize);
                        }

                        let prev_len = compact_int_vec.len();

                        for i in range.clone() {
                            compact_int_vec.set(i, 2_usize.pow(i as u32));
                        }

                        assert_eq!(compact_int_vec.len(), prev_len);

                        for i in range {
                            assert_eq!(compact_int_vec.get(i).unwrap(), 2_usize.pow(i as u32));
                        }
                    }

                    #[test]
                    fn len() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);

                        assert_eq!(compact_int_vec.len(), 0);

                        compact_int_vec.push(MIN_VALUE);
                        compact_int_vec.push(MAX_VALUE);

                        assert_eq!(compact_int_vec.len(), 2);
                    }

                    #[test]
                    fn is_empty() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        assert!(compact_int_vec.is_empty());

                        compact_int_vec.push(MIN_VALUE);
                        compact_int_vec.push(MAX_VALUE);

                        assert!(!compact_int_vec.is_empty());
                    }

                    // Iterator tests
                    #[test]
                    fn into_iter() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);

                        compact_int_vec.push(MIN_VALUE);
                        compact_int_vec.push(MID_VALUE);
                        compact_int_vec.push(MAX_VALUE);

                        let mut iter = compact_int_vec.into_iter();
                        assert_eq!(iter.next(), Some(MIN_VALUE));
                        assert_eq!(iter.next(), Some(MID_VALUE));
                        assert_eq!(iter.next(), Some(MAX_VALUE));
                        assert_eq!(iter.next(), None);
                    }

                    #[test]
                    fn iter() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);

                        compact_int_vec.push(MIN_VALUE);
                        compact_int_vec.push(MID_VALUE);
                        compact_int_vec.push(MAX_VALUE);

                        let mut iter = compact_int_vec.iter();
                        assert_eq!(iter.next(), Some(MIN_VALUE));
                        assert_eq!(iter.next(), Some(MID_VALUE));
                        assert_eq!(iter.next(), Some(MAX_VALUE));
                        assert_eq!(iter.next(), None);
                    }

                    #[test]
                    fn for_loop() {
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);

                        for _ in 0..=10 {
                            compact_int_vec.push(MAX_VALUE);
                        }

                        for value in compact_int_vec {
                            assert_eq!(value, MAX_VALUE);
                        }
                    }

                    #[test]
                    fn extend(){
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        let values = [MIN_VALUE, MID_VALUE, MAX_VALUE];

                        compact_int_vec.extend(values.iter().copied());

                        assert_eq!(compact_int_vec.len(), 3);
                        assert_eq!(compact_int_vec.get(0).unwrap(), MIN_VALUE);
                        assert_eq!(compact_int_vec.get(1).unwrap(), MID_VALUE);
                        assert_eq!(compact_int_vec.get(2).unwrap(), MAX_VALUE);
                        assert_eq!(compact_int_vec.width(), WIDTH);
                    }

                    #[test]
                    #[should_panic]
                    fn extend_fails_when_value_does_not_fit(){
                        let mut compact_int_vec = CompactIntVec::new(WIDTH);
                        let values = [MIN_VALUE, MID_VALUE, MAX_VALUE+1];

                        compact_int_vec.extend(values.iter().copied());
                    }
                }
            }
        )*
    }
}

macro_rules! test_from_for_type {
    ($($t:ty),*) => {
        $(
            paste!{
               #[test]
                fn [<from_ $t _slice>]() {
                    let values = [$t::MIN, $t::MAX/2, $t::MAX];
                    let compact_int_vec = CompactIntVec::from(&values[..]);

                    assert_eq!(compact_int_vec.len(), 3);
                    // width should be inferred, and as we use $t::MAX, it should be the bits that $t can hold
                    assert_eq!(compact_int_vec.width(), std::mem::size_of::<$t>() * 8);
                    assert_eq!(compact_int_vec.get(0).unwrap(), $t::MIN as usize);
                    assert_eq!(compact_int_vec.get(1).unwrap(), $t::MAX as usize / 2);
                    assert_eq!(compact_int_vec.get(2).unwrap(), $t::MAX as usize);
                }

                #[test]
                #[should_panic]
                fn [<from_empty_ $t _slice_fails>]() {
                    let values:[$t;0] = [];
                    let _ = CompactIntVec::from(&values[..]);
                }

                #[test]
                fn [<from_ $t _vec>]() {
                    let values = vec![$t::MIN, $t::MAX/2, $t::MAX];
                    let compact_int_vec = CompactIntVec::from(values);

                    assert_eq!(compact_int_vec.len(), 3);
                    // width should be inferred, and as we use $t::MAX, it should be the bits that $t can hold
                    assert_eq!(compact_int_vec.width(), std::mem::size_of::<$t>() * 8);
                    assert_eq!(compact_int_vec.get(0).unwrap(), $t::MIN as usize);
                    assert_eq!(compact_int_vec.get(1).unwrap(), $t::MAX as usize / 2);
                    assert_eq!(compact_int_vec.get(2).unwrap(), $t::MAX as usize);
                }

                #[test]
                #[should_panic]
                fn [<from_empty_ $t _vec_fails>]() {
                    let values:Vec<$t> = vec![];
                    let _ = CompactIntVec::from(values);
                }

                #[test]
                fn [<from_ $t _array>]() {
                    let values = [$t::MIN, $t::MAX/2, $t::MAX];
                    let compact_int_vec = CompactIntVec::from(values);

                    assert_eq!(compact_int_vec.len(), 3);
                    // width should be inferred, and as we use $t::MAX, it should be the bits that $t can hold
                    assert_eq!(compact_int_vec.width(), std::mem::size_of::<$t>() * 8);
                    assert_eq!(compact_int_vec.get(0).unwrap(), $t::MIN as usize);
                    assert_eq!(compact_int_vec.get(1).unwrap(), $t::MAX as usize / 2);
                    assert_eq!(compact_int_vec.get(2).unwrap(), $t::MAX as usize);
                }

                #[test]
                #[should_panic]
                fn [<from_empty_ $t _array_fails>]() {
                    let values:[$t;0] = [];
                    let _ = CompactIntVec::from(values);
                }

                #[test]
                fn [<from_ $t _ref_array>]() {
                    let values = &[$t::MIN, $t::MAX/2, $t::MAX];
                    let compact_int_vec = CompactIntVec::from(values);

                    assert_eq!(compact_int_vec.len(), 3);
                    // width should be inferred, and as we use $t::MAX, it should be the bits that $t can hold
                    assert_eq!(compact_int_vec.width(), std::mem::size_of::<$t>() * 8);
                    assert_eq!(compact_int_vec.get(0).unwrap(), $t::MIN as usize);
                    assert_eq!(compact_int_vec.get(1).unwrap(), $t::MAX as usize / 2);
                    assert_eq!(compact_int_vec.get(2).unwrap(), $t::MAX as usize);
                }

                #[test]
                #[should_panic]
                fn [<from_empty_ $t _ref_array_fails>]() {
                    let values:&[$t;0] = &[];
                    let _ = CompactIntVec::from(values);
                }


                #[test]
                fn [<from_ $t _iterator>]() {
                    let values = [$t::MIN, $t::MAX/2, $t::MAX];
                    let compact_int_vec = values.into_iter().collect::<CompactIntVec>();

                    assert_eq!(compact_int_vec.len(), 3);
                    // width should be inferred, and as we use $t::MAX, it should be the bits that $t can hold
                    assert_eq!(compact_int_vec.width(), std::mem::size_of::<$t>() * 8);
                    assert_eq!(compact_int_vec.get(0).unwrap(), $t::MIN as usize);
                    assert_eq!(compact_int_vec.get(1).unwrap(), $t::MAX as usize / 2);
                    assert_eq!(compact_int_vec.get(2).unwrap(), $t::MAX as usize);
                }

                #[test]
                #[should_panic]
                fn [<from_empty_ $t _iterator_fails>]() {
                    let values:[$t;0] = [];
                    let _ = values.into_iter().collect::<CompactIntVec>();
                }

            }
        )*

    }
}

test_for_width!(0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 16);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
test_for_width!(17, 18, 20, 25, 31, 32);

// Do not test for 64 since it would cause errors with MAX_VALUE+1 tests
#[cfg(target_pointer_width = "64")]
test_for_width!(33, 40, 48, 63);

test_from_for_type!(u8, u16, usize);
