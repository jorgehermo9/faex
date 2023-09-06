macro_rules! test_rank_for {
    ($t:ty, $($args:tt)*) => {
        mod rank{
            use super::*;
            use crate::bit_vectors::rank_select::Rank;

            const WORD_SIZE: usize = std::mem::size_of::<usize>() * 8;
            const BIG_BITVEC_SIZE: usize = 10_000;

            #[test]
            fn rank() {
                let bv = BitVec::from([0b10101010u8;8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.rank(0), Some(0));
                assert_eq!(rs.rank(1), Some(0));
                assert_eq!(rs.rank(2), Some(1));
                assert_eq!(rs.rank(3), Some(1));
                assert_eq!(rs.rank(4), Some(2));
                assert_eq!(rs.rank(5), Some(2));
                assert_eq!(rs.rank(6), Some(3));
                assert_eq!(rs.rank(7), Some(3));
                assert_eq!(rs.rank(8), Some(4));
                assert_eq!(rs.rank(64), Some(32));
                assert_eq!(rs.rank(65), None);
            }

            #[test]
            fn rank_when_empty_bitvec(){
                let bv = BitVec::new();
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.rank(0), Some(0));
                assert_eq!(rs.rank(1), None);
            }

            #[test]
            fn rank_when_data_spans_more_than_one_word() {
                const TEST_DATA: [usize; 10] = [
                    0b1000, 0b0010, 0b0000, 0b0110, 0b0000, 0b1010, 0b0000, 0b1011, 0b0100, 0b0001,
                ];

                let bv = BitVec::from(TEST_DATA);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.rank(0), Some(0));
                assert_eq!(rs.rank(1), Some(0));
                assert_eq!(rs.rank(2), Some(0));
                assert_eq!(rs.rank(3), Some(0));
                assert_eq!(rs.rank(4), Some(1));
                assert_eq!(rs.rank(WORD_SIZE + 1), Some(1));
                assert_eq!(rs.rank(WORD_SIZE + 2), Some(2));
                assert_eq!(rs.rank(WORD_SIZE + 3), Some(2));
                assert_eq!(rs.rank(WORD_SIZE + 4), Some(2));
                assert_eq!(rs.rank(WORD_SIZE * 2 + 1), Some(2));
                assert_eq!(rs.rank(WORD_SIZE * 2 + 2), Some(2));
                assert_eq!(rs.rank(WORD_SIZE * 2 + 3), Some(2));
                assert_eq!(rs.rank(WORD_SIZE * 2 + 4), Some(2));
                assert_eq!(rs.rank(WORD_SIZE * 3 + 1), Some(2));
                assert_eq!(rs.rank(WORD_SIZE * 3 + 2), Some(3));
                assert_eq!(rs.rank(WORD_SIZE * 3 + 3), Some(4));
                assert_eq!(rs.rank(WORD_SIZE * 3 + 4), Some(4));
                assert_eq!(rs.rank(WORD_SIZE * 4 + 1), Some(4));
                assert_eq!(rs.rank(WORD_SIZE * 4 + 2), Some(4));
                assert_eq!(rs.rank(WORD_SIZE * 4 + 3), Some(4));
                assert_eq!(rs.rank(WORD_SIZE * 4 + 4), Some(4));
                assert_eq!(rs.rank(WORD_SIZE * 5 + 1), Some(4));
                assert_eq!(rs.rank(WORD_SIZE * 5 + 2), Some(5));
                assert_eq!(rs.rank(WORD_SIZE * 5 + 3), Some(5));
                assert_eq!(rs.rank(WORD_SIZE * 5 + 4), Some(6));
            }

            #[test]
            fn rank_when_data_spans_less_than_a_word(){
                let bv = BitVec::from([0b10101010u8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.rank(0), Some(0));
                assert_eq!(rs.rank(1), Some(0));
                assert_eq!(rs.rank(2), Some(1));
                assert_eq!(rs.rank(3), Some(1));
                assert_eq!(rs.rank(4), Some(2));
                assert_eq!(rs.rank(5), Some(2));
                assert_eq!(rs.rank(6), Some(3));
                assert_eq!(rs.rank(7), Some(3));
                assert_eq!(rs.rank(8), Some(4));
                assert_eq!(rs.rank(9), None);
            }

            #[test]
            fn rank_when_all_ones() {
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.rank(i), Some(i));
                }
            }

            #[test]
            fn rank_when_all_zeros() {
                let bv = BitVec::from_value(false, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.rank(i), Some(0));
                }
            }

            #[test]
            fn rank_when_index_is_out_of_bounds() {
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.rank(BIG_BITVEC_SIZE + 1), None);
            }

            #[test]
            fn rank_with_random_values(){
                let mut bv = BitVec::new();
                for _ in 0..BIG_BITVEC_SIZE {
                    bv.push(rand::random());
                }
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv.clone());

                let mut acc = 0;
                for i in 0..BIG_BITVEC_SIZE {
                    if bv.read(i) {
                        assert_eq!(rs.rank(i), Some(acc));
                        acc += 1;
                    } else {
                        assert_eq!(rs.rank(i), Some(acc));
                    }
                }
            }

            #[test]
            fn rank0() {
                let bv = BitVec::from([0b10101010u8; 8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                assert_eq!(rs.rank0(0), Some(0));
                assert_eq!(rs.rank0(1), Some(1));
                assert_eq!(rs.rank0(2), Some(1));
                assert_eq!(rs.rank0(3), Some(2));
                assert_eq!(rs.rank0(4), Some(2));
                assert_eq!(rs.rank0(5), Some(3));
                assert_eq!(rs.rank0(6), Some(3));
                assert_eq!(rs.rank0(7), Some(4));
                assert_eq!(rs.rank0(8), Some(4));
                assert_eq!(rs.rank(64), Some(32));
                assert_eq!(rs.rank(65), None);

            }

            #[test]
            fn rank0_when_empty_bitvec(){
                let bv = BitVec::new();
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                assert_eq!(rs.rank0(0), Some(0));
                assert_eq!(rs.rank0(1), None);
            }

            #[test]
            fn rank0_when_data_spans_more_than_one_word() {
                const TEST_DATA: [usize; 10] = [
                    0b1000, 0b0010, 0b0000, 0b0110, 0b0000, 0b1010, 0b0000, 0b1011, 0b0100, 0b0001,
                ];

                let bv = BitVec::from(TEST_DATA);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                assert_eq!(rs.rank0(0), Some(0));
                assert_eq!(rs.rank0(1), Some(1));
                assert_eq!(rs.rank0(2), Some(2));
                assert_eq!(rs.rank0(3), Some(3));
                assert_eq!(rs.rank0(4), Some(3));
                assert_eq!(rs.rank0(WORD_SIZE + 1), Some(4 + 60));
                assert_eq!(rs.rank0(WORD_SIZE + 2), Some(4 + 60));
                assert_eq!(rs.rank0(WORD_SIZE + 3), Some(5 + 60));
                assert_eq!(rs.rank0(WORD_SIZE + 4), Some(6 + 60));
                assert_eq!(rs.rank0(WORD_SIZE * 2 + 1), Some(7 + 60*2));
                assert_eq!(rs.rank0(WORD_SIZE * 2 + 2), Some(8 + 60*2));
                assert_eq!(rs.rank0(WORD_SIZE * 2 + 3), Some(9 + 60*2));
                assert_eq!(rs.rank0(WORD_SIZE * 2 + 4), Some(10 + 60*2));
                assert_eq!(rs.rank0(WORD_SIZE * 3 + 1), Some(11 + 60*3));
                assert_eq!(rs.rank0(WORD_SIZE * 3 + 2), Some(11 + 60*3));
                assert_eq!(rs.rank0(WORD_SIZE * 3 + 3), Some(11 + 60*3));
                assert_eq!(rs.rank0(WORD_SIZE * 3 + 4), Some(12 + 60*3));
                assert_eq!(rs.rank0(WORD_SIZE * 4 + 1), Some(13 + 60*4));
                assert_eq!(rs.rank0(WORD_SIZE * 4 + 2), Some(14 + 60*4));
                assert_eq!(rs.rank0(WORD_SIZE * 4 + 3), Some(15 + 60*4));
                assert_eq!(rs.rank0(WORD_SIZE * 4 + 4), Some(16 + 60*4));
                assert_eq!(rs.rank0(WORD_SIZE * 5 + 1), Some(17 + 60*5));
                assert_eq!(rs.rank0(WORD_SIZE * 5 + 2), Some(17 + 60*5));
                assert_eq!(rs.rank0(WORD_SIZE * 5 + 3), Some(18 + 60*5));
                assert_eq!(rs.rank0(WORD_SIZE * 5 + 4), Some(18 + 60*5));
            }

            #[test]
            fn rank0_when_data_spans_less_than_a_word(){
                let bv = BitVec::from([0b10101010u8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                assert_eq!(rs.rank0(0), Some(0));
                assert_eq!(rs.rank0(1), Some(1));
                assert_eq!(rs.rank0(2), Some(1));
                assert_eq!(rs.rank0(3), Some(2));
                assert_eq!(rs.rank0(4), Some(2));
                assert_eq!(rs.rank0(5), Some(3));
                assert_eq!(rs.rank0(6), Some(3));
                assert_eq!(rs.rank0(7), Some(4));
                assert_eq!(rs.rank0(8), Some(4));
                assert_eq!(rs.rank0(9), None);
            }

            #[test]
            fn rank0_when_all_ones() {
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.rank0(i), Some(0));
                }
            }

            #[test]
            fn rank0_when_all_zeros() {
                let bv = BitVec::from_value(false, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);



                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.rank0(i), Some(i));
                }
            }

            #[test]
            fn rank0_with_random_values(){
                let mut bv = BitVec::new();
                for _ in 0..BIG_BITVEC_SIZE {
                    bv.push(rand::random());
                }
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv.clone());

                let mut acc = 0;
                for i in 0..BIG_BITVEC_SIZE {
                    if bv.read(i) {
                        assert_eq!(rs.rank0(i), Some(acc));
                    } else {
                        assert_eq!(rs.rank0(i), Some(acc));
                        acc += 1;
                    }
                }
            }

            #[test]
            fn rank0_when_index_is_out_of_bounds() {
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                assert_eq!(rs.rank0(BIG_BITVEC_SIZE + 1), None);
            }
        }
    };
}

macro_rules! test_select_for {
    ($t:ty, $($args:tt)*) => {
        mod select {
            use super::*;
            use crate::bit_vectors::rank_select::Select;
            const WORD_SIZE: usize = std::mem::size_of::<usize>() * 8;
            const BIG_BITVEC_SIZE: usize = 10_000;

            #[test]
            fn select() {
                let bv = BitVec::from([0b10101010u8;8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select(0), Some(0));
                assert_eq!(rs.select(1), Some(2));
                assert_eq!(rs.select(2), Some(4));
                assert_eq!(rs.select(3), Some(6));
                assert_eq!(rs.select(4), Some(8));
                assert_eq!(rs.select(32), Some(64));
                assert_eq!(rs.select(33), None);
            }

            #[test]
            fn select_when_empty_bitvec(){
                let bv = BitVec::new();
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select(0), Some(0));
                assert_eq!(rs.select(1), None);
            }

            #[test]
            fn select_when_data_spans_more_than_one_word() {
                const TEST_DATA: [usize; 10] = [
                    0b1000, 0b0010, 0b0000, 0b0110, 0b0000, 0b1010, 0b0000, 0b1011, 0b0100, 0b0001,
                ];
                let bv = BitVec::from(TEST_DATA);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select(0), Some(0));
                assert_eq!(rs.select(1), Some(4));
                assert_eq!(rs.select(2), Some(WORD_SIZE + 2));
                assert_eq!(rs.select(3), Some(WORD_SIZE*3 + 2));
                assert_eq!(rs.select(4), Some(WORD_SIZE*3 + 3));
                assert_eq!(rs.select(5), Some(WORD_SIZE*5 + 2));
                assert_eq!(rs.select(6), Some(WORD_SIZE*5 + 4));
                assert_eq!(rs.select(7), Some(WORD_SIZE*7 + 1));
                assert_eq!(rs.select(8), Some(WORD_SIZE*7 + 2));
                assert_eq!(rs.select(9), Some(WORD_SIZE*7 + 4));
                assert_eq!(rs.select(10), Some(WORD_SIZE*8 + 3));
                assert_eq!(rs.select(11), Some(WORD_SIZE*9 + 1));
            }

            // TODO: test select when data spans less than a word

            #[test]
            fn select_when_data_spans_less_than_a_word(){
                let bv = BitVec::from([0b10101010u8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select(0), Some(0));
                assert_eq!(rs.select(1), Some(2));
                assert_eq!(rs.select(2), Some(4));
                assert_eq!(rs.select(3), Some(6));
                assert_eq!(rs.select(4), Some(8));
                assert_eq!(rs.select(5), None);
            }

            #[test]
            fn select_when_all_ones() {
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                for i in 0..=BIG_BITVEC_SIZE {
                    assert_eq!(rs.select(i), Some(i));
                }
            }

            #[test]
            fn select_when_all_zeros() {
                let bv = BitVec::from_value(false, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select(0), Some(0));
                for i in 1..BIG_BITVEC_SIZE {
                    assert_eq!(rs.select(i), None);
                }

                assert_eq!(rs.select(BIG_BITVEC_SIZE + 1), None);
            }

            #[test]
            fn select_with_random_values(){
                let mut bv = BitVec::new();
                for _ in 0..BIG_BITVEC_SIZE {
                    bv.push(rand::random());
                }
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv.clone());

                let mut acc = 1;
                for i in 0..BIG_BITVEC_SIZE {
                    if bv.read(i) {
                        assert_eq!(rs.select(acc), Some(i+1));
                        acc += 1;
                    }
                }
            }

            #[test]
            fn select0() {
                let bv = BitVec::from([0b10101010u8;8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select0(0), Some(0));
                assert_eq!(rs.select0(1), Some(1));
                assert_eq!(rs.select0(2), Some(3));
                assert_eq!(rs.select0(3), Some(5));
                assert_eq!(rs.select0(4), Some(7));
                assert_eq!(rs.select0(32), Some(63));
                assert_eq!(rs.select0(33), None);
            }

            #[test]
            fn select0_when_empty_bitvec(){
                let bv = BitVec::new();
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select0(0), Some(0));
                assert_eq!(rs.select0(1), None);
            }

            #[test]
            fn select0_when_all_ones(){
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.select0(0), Some(0));
                for i in 1..BIG_BITVEC_SIZE {
                    assert_eq!(rs.select0(i), None);
                }

                assert_eq!(rs.select0(BIG_BITVEC_SIZE + 1), None);
            }

            #[test]
            fn select0_when_all_zeros(){
                let bv = BitVec::from_value(false, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                for i in 0..=BIG_BITVEC_SIZE {
                    assert_eq!(rs.select0(i), Some(i));
                }
            }

            #[test]
            fn select0_with_random_values(){
                let mut bv = BitVec::new();
                for _ in 0..BIG_BITVEC_SIZE {
                    bv.push(rand::random());
                }
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv.clone());

                let mut acc = 1;
                for i in 0..BIG_BITVEC_SIZE {
                    if !bv.read(i) {
                        assert_eq!(rs.select0(acc), Some(i+1));
                        acc += 1;
                    }
                }
            }

        }
    };
}

macro_rules! test_access_for {
    ($t:ty, $($args:tt)*) => {
        mod access {
            use super::*;
            use crate::bit_vectors::Access;
            const BIG_BITVEC_SIZE: usize = 10_000;


            #[test]
            fn access_when_all_ones(){
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.access(i), Some(true));
                }
            }

            #[test]
            fn access_when_all_zeros(){
                let bv = BitVec::from_value(false, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.access(i), Some(false));
                }
            }

            #[test]
            fn access_with_random_values(){
                let mut bv = BitVec::new();
                for _ in 0..BIG_BITVEC_SIZE {
                    bv.push(rand::random());
                }
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv.clone());

                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.access(i), Some(bv.read(i)));
                }
            }

            #[test]
            fn access_when_empty_bitvec(){
                let bv = BitVec::new();
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.access(0), None);
            }

            #[test]
            fn access_when_index_is_out_of_bounds(){
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);

                assert_eq!(rs.access(BIG_BITVEC_SIZE + 1), None);
            }

        }
    };
}

macro_rules! test_rank_select_for{
    ($t:ty, $($args:tt)*) => {
        mod rank_select{
            use super::*;
            use crate::bit_vectors::rank_select::tests_utils::{test_rank_for, test_select_for};
            const BIG_BITVEC_SIZE: usize = 10_000;

            test_rank_for!($t, $($args)*);
            test_select_for!($t, $($args)*);

            #[test]
            fn rank_is_the_inverse_of_select(){
                use crate::bit_vectors::rank_select::{Rank,Select};

                let bv = BitVec::from([0b10101010u8;8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                for i in 0..=32 {
                    assert_eq!(rs.rank(rs.select(i).unwrap()), Some(i));
                }
            }

            #[test]
            fn rank_is_the_inverse_of_select_when_all_ones(){
                use crate::bit_vectors::rank_select::{Rank,Select};
                let bv = BitVec::from_value(true, BIG_BITVEC_SIZE);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                for i in 0..BIG_BITVEC_SIZE {
                    assert_eq!(rs.rank(rs.select(i).unwrap()), Some(i));
                }
            }

            #[test]
            fn rank_is_the_inverse_of_select_with_random_values(){
                use crate::bit_vectors::rank_select::{Rank,Select};
                let mut bv = BitVec::new();
                for _ in 0..BIG_BITVEC_SIZE {
                    bv.push(rand::random());
                }
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv.clone());

                let mut acc = 0;
                for i in 0..BIG_BITVEC_SIZE {
                    if bv.read(i) {
                        assert_eq!(rs.rank(rs.select(acc).unwrap()), Some(acc));
                        acc += 1;
                    }
                }
            }

            #[test]
            fn rank0_is_the_inverse_of_select0(){
                use crate::bit_vectors::rank_select::{Rank,Select};

                let bv = BitVec::from([0b10101010u8;8]);
                let spec = <$t>::spec($($args)*);
                let rs = spec.build(bv);


                for i in 0..=32 {
                    assert_eq!(rs.rank0(rs.select0(i).unwrap()), Some(i));
                }
            }

        }
    }
}

macro_rules! test_rank_select_access_for{
    ($t:ty, $($args:tt)*) => {
        use crate::bit_vectors::rank_select::tests_utils::{test_rank_select_for,test_access_for};

        test_rank_select_for!($t, $($args)*);
        test_access_for!($t, $($args)*);
    }
}

pub(crate) use test_access_for;
pub(crate) use test_rank_for;
pub(crate) use test_rank_select_access_for;
pub(crate) use test_rank_select_for;
pub(crate) use test_select_for;
