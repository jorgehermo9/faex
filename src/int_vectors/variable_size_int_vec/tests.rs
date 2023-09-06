use super::*;
use crate::util::ceil_div;
use paste::paste;

// The test below cannot be inside macro since its samples values are hardcoded and depend on K
const K: usize = 4;
#[test]
fn new() {
    let l = [0, 1, 2, 3, 4];

    let size_fn = |index| l.get(index).copied().unwrap_or(0);

    let int_vec = VariableSizeIntVec::new(size_fn, K);

    assert_eq!(int_vec.len(), 0);
    assert_eq!(int_vec.k(), 4);
    assert_eq!(int_vec.samples().len(), 0);
}
// TODO: push fail when size > usize::BITS

#[test]
fn push() {
    let l = [0, 1, 2, 3, 4];
    let values: [usize; 5] = [0, 1, 2, 7, 10];

    let size_fn = |index| l.get(index).copied().unwrap_or(0);
    let mut int_vec = VariableSizeIntVec::new(size_fn, K);

    for value in values {
        int_vec.push(value);
    }

    assert_eq!(int_vec.len(), 5);
    assert_eq!(int_vec.raw_data().len(), 10);
    assert_eq!(*int_vec.samples.get(0).unwrap(), 0);
    assert_eq!(*int_vec.samples.get(1).unwrap(), 6);

    assert_eq!(int_vec.get(0), 0);
    assert_eq!(int_vec.get(1), 1);
    assert_eq!(int_vec.get(2), 2);
    assert_eq!(int_vec.get(3), 7);
    assert_eq!(int_vec.get(4), 10);
}

#[test]
fn push_when_sizes_are_not_sorted() {
    let l = [1usize, 1, 4, 4, 2, 3, 1, 0, 0];
    let values = l
        .iter()
        .map(|x| 2usize.pow(*x as u32) - 1)
        .collect::<Vec<_>>();

    let size_fn = |index| l.get(index).copied().unwrap_or(0);
    let mut int_vec = VariableSizeIntVec::new(size_fn, K);

    for value in &values {
        int_vec.push(*value);
    }

    assert_eq!(int_vec.len(), 9);
    assert_eq!(int_vec.raw_data().len(), 16);
    assert_eq!(*int_vec.samples.get(0).unwrap(), 0);
    assert_eq!(*int_vec.samples.get(1).unwrap(), 10);
    assert_eq!(*int_vec.samples.get(2).unwrap(), 16);

    for (i, value) in values.iter().enumerate() {
        assert_eq!(int_vec.get(i), *value);
    }
}

#[test]
fn pop() {
    let l = [0, 1, 2, 3, 4];
    let values: [usize; 5] = [0, 1, 2, 7, 10];

    let size_fn = |index| l.get(index).copied().unwrap_or(0);
    let mut int_vec = VariableSizeIntVec::new(size_fn, K);

    for value in &values {
        int_vec.push(*value);
    }

    assert_eq!(int_vec.len(), 5);
    assert_eq!(int_vec.raw_data().len(), 10);
    assert_eq!(*int_vec.samples.get(0).unwrap(), 0);
    assert_eq!(*int_vec.samples.get(1).unwrap(), 6);

    assert_eq!(int_vec.pop(), Some(10));
    assert_eq!(int_vec.pop(), Some(7));
    assert_eq!(int_vec.pop(), Some(2));
    assert_eq!(int_vec.pop(), Some(1));
    assert_eq!(int_vec.pop(), Some(0));
    assert_eq!(int_vec.pop(), None);

    assert_eq!(int_vec.len(), 0);
    assert_eq!(int_vec.raw_data().len(), 0);
    assert_eq!(int_vec.samples().len(), 0);
}

#[test]
fn set() {
    let l = [0, 1, 2, 3, 4];
    let values: [usize; 5] = [0, 1, 2, 7, 10];

    let size_fn = |index| l.get(index).copied().unwrap_or(0);
    let mut int_vec = VariableSizeIntVec::new(size_fn, K);

    for value in &values {
        int_vec.push(*value);
    }

    assert_eq!(int_vec.len(), 5);
    assert_eq!(int_vec.raw_data().len(), 10);
    assert_eq!(*int_vec.samples.get(0).unwrap(), 0);
    assert_eq!(*int_vec.samples.get(1).unwrap(), 6);

    int_vec.set(0, 0usize);
    int_vec.set(1, 0usize);
    int_vec.set(2, 0b11usize);
    int_vec.set(3, 0b010usize);
    int_vec.set(4, 0b0110usize);

    assert_eq!(int_vec.len(), 5);
    assert_eq!(*int_vec.samples.get(0).unwrap(), 0);
    assert_eq!(*int_vec.samples.get(1).unwrap(), 6);

    assert_eq!(int_vec.get(0), 0);
    assert_eq!(int_vec.get(1), 0);
    assert_eq!(int_vec.get(2), 0b11);
    assert_eq!(int_vec.get(3), 0b010);
    assert_eq!(int_vec.get(4), 0b0110);
}

macro_rules! test_for_k{
    ($( $k: expr ),*) => {
        $(
            paste!{
                mod [<when_k_is_ $k>]{
                    use super::*;
                    const K: usize = $k;

                    #[test]
                    fn push_when_n_is_large() {
                        let n = 1000;
                        let size_fn = |index: usize| index.checked_ilog2().map(|v| v + 1).unwrap_or(0) as usize;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);

                        for i in 0..n {
                            int_vec.push(i);
                        }

                        assert_eq!(int_vec.len(), n);
                        assert_eq!(int_vec.raw_data().len(), 8977);

                        for i in 0..n {
                            assert_eq!(int_vec.get(i), i);
                        }
                    }

                    #[test]
                    fn push_when_n_is_large_and_ints_are_zero_sized() {
                        let n = 1000;
                        let size_fn = |_| 0;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        for _ in 0..n {
                            int_vec.push(0usize);
                        }
                        assert_eq!(int_vec.len(), n);
                        assert_eq!(int_vec.raw_data().len(), 0);

                        for i in 0..n {
                            assert_eq!(int_vec.get(i), 0);
                        }
                    }

                    #[test]
                    #[should_panic]
                    fn push_fails_when_value_does_not_fit() {
                        let size_fn = |_| 0;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        int_vec.push(1usize);
                    }

                    #[test]
                    #[should_panic]
                    fn get_fails_when_index_out_of_bounds() {
                        let size_fn = |_| 0;
                        let int_vec = VariableSizeIntVec::new(size_fn, K);
                        int_vec.get(0);
                    }

                    #[test]
                    fn set_when_n_is_large(){
                        let n = 1000;
                        let size_fn = |index: usize| index.checked_ilog2().map(|v| v + 1).unwrap_or(0) as usize;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        for i in 0..n {
                            int_vec.push(i);
                        }

                        assert_eq!(int_vec.len(), n);
                        assert_eq!(int_vec.raw_data().len(), 8977);

                        for i in 0..n {
                            int_vec.set(i, 0usize);
                        }

                        assert_eq!(int_vec.len(), n);
                        assert_eq!(int_vec.raw_data().len(), 8977);

                        for i in 0..n {
                            assert_eq!(int_vec.get(i), 0);
                        }
                    }

                    #[test]
                    #[should_panic]
                    fn set_fails_when_value_does_not_fit() {
                        let size_fn = |_| 0;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        int_vec.push(0usize);
                        int_vec.set(0, 1usize);
                    }

                    #[test]
                    #[should_panic]
                    fn set_fails_when_index_out_of_bounds() {
                        let size_fn = |_| 0;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        int_vec.set(0, 0usize);
                    }

                    #[test]
                    fn heap_size_in_bits() {
                        const WORD_SIZE: usize = std::mem::size_of::<usize>() * 8;
                        let n = 1000usize;
                        let size_fn = |index: usize| index.checked_ilog2().unwrap_or(0) as usize + 1;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);

                        for i in 0..n {
                            int_vec.push(i);
                        }

                        let num_samples = ceil_div(n,K);
                        let sample_overhead = WORD_SIZE as usize * num_samples;
                        // Samples are stored as arbitrary-width integers, so the total number
                        // of bits used may be not a multiple of the underlying storage (usize)
                        let sample_overhead = ceil_div(sample_overhead,WORD_SIZE) * WORD_SIZE;
                        let mut acc_sizes = (0..n).map(&size_fn).sum::<usize>();
                        if acc_sizes % WORD_SIZE != 0 {
                            // round up to the nearest word, since bitmap is word-aligned
                            acc_sizes += WORD_SIZE - (acc_sizes % WORD_SIZE);
                        }

                        let expected_size = sample_overhead + acc_sizes;
                        assert_eq!(int_vec.heap_size_in_bits(), expected_size);
                    }

                    // Iterator tests
                    // Iterator tests
                    #[test]
                    fn into_iter() {
                        let n = 1000;
                        let size_fn = |index: usize| index.checked_ilog2().map(|v| v + 1).unwrap_or(0) as usize;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        for i in 0..n {
                            int_vec.push(i);
                        }

                        assert_eq!(int_vec.len(), n);
                        assert_eq!(int_vec.raw_data().len(), 8977);

                        let mut iter = int_vec.into_iter();
                        for i in 0..n {
                            assert_eq!(iter.next(), Some(i));
                        }
                        assert_eq!(iter.next(), None);
                    }

                    #[test]
                    fn iter() {
                        let n = 1000;
                        let size_fn = |index: usize| index.checked_ilog2().map(|v| v + 1).unwrap_or(0) as usize;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        for i in 0..n {
                            int_vec.push(i);
                        }

                        assert_eq!(int_vec.len(), n);
                        assert_eq!(int_vec.raw_data().len(), 8977);

                        let mut iter = int_vec.iter();
                        for i in 0..n {
                            assert_eq!(iter.next(), Some(i));
                        }
                        assert_eq!(iter.next(), None);
                    }

                    #[test]
                    fn for_loop() {
                        let n = 1000usize;
                        let size_fn = |index: usize| index.checked_ilog2().map(|v| v + 1).unwrap_or(0) as usize;
                        let mut int_vec = VariableSizeIntVec::new(size_fn, K);
                        for i in 0..n {
                            int_vec.push(i);
                        }

                        let mut acc =  0;
                        for value in int_vec {
                            assert_eq!(value, acc);
                            acc+=1;
                        }
                    }
                }
            }
        )*
    }
}

test_for_k!(1, 2, 3, 4, 8, 15, 16, 31, 32, 63, 64, 128, 256, 512);
