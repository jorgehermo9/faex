use std::collections::{BTreeSet, HashMap};

use super::*;
use crate::bit_vectors::bitvec::BitVecSpec;
use crate::bit_vectors::rank_select::dense_sampling_rank::{
    DenseSamplingRank, DenseSamplingRankSpec,
};
use crate::bit_vectors::rank_select::sparse_sampling_rank::{
    SparseSamplingRank, SparseSamplingRankSpec,
};
use crate::bit_vectors::rrr_bitvec::{RRRBitVec, RRRBitVecSpec};

fn build_test_wt() -> WaveletTreeNode<BitVec> {
    // Build the following wt root node:
    // let data = "tobeornottobethatisthequestion".to_string();
    // let spec = BitVec::spec();
    // let wt = WaveletTree::new(data.clone(), spec);

    let a_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 1 };
    let b_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 2 };
    let e_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 4 };
    let h_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 2 };
    let i_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 2 };
    let n_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 2 };
    let o_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 5 };
    let q_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 1 };
    let r_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 1 };
    let s_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 2 };
    let t_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 7 };
    let u_leaf = WaveletTreeNode::<BitVec>::Leaf { len: 1 };

    let a_b_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(a_leaf),
        right: Box::new(b_leaf),
        bit_vec: BitVec::from([true, true, false]),
    };

    let b_e_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(a_b_internal),
        right: Box::new(e_leaf),
        bit_vec: BitVec::from([false, true, false, true, false, true, true]),
    };

    let h_i_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(h_leaf),
        right: Box::new(i_leaf),
        bit_vec: BitVec::from([false, true, false, true]),
    };

    let h_n_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(h_i_internal),
        right: Box::new(n_leaf),
        bit_vec: BitVec::from([true, false, false, false, false, true]),
    };

    let b_n_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(b_e_internal),
        right: Box::new(h_n_internal),
        bit_vec: BitVec::from([
            false, false, true, false, false, true, false, true, true, false, false, true, true,
        ]),
    };

    let o_q_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(o_leaf),
        right: Box::new(q_leaf),
        bit_vec: BitVec::from([false, false, false, false, true, false]),
    };

    let o_r_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(o_q_internal),
        right: Box::new(r_leaf),
        bit_vec: BitVec::from([false, false, true, false, false, false, false]),
    };

    let s_t_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(s_leaf),
        right: Box::new(t_leaf),
        bit_vec: BitVec::from([true, true, true, true, true, false, true, false, true]),
    };

    let s_u_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(s_t_internal),
        right: Box::new(u_leaf),
        bit_vec: BitVec::from([
            false, false, false, false, false, false, false, true, false, false,
        ]),
    };

    let o_u_internal = WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(o_r_internal),
        right: Box::new(s_u_internal),
        bit_vec: BitVec::from([
            true, false, false, false, false, true, true, false, true, true, true, true, false,
            true, true, true, false,
        ]),
    };

    WaveletTreeNode::<BitVec>::Internal {
        left: Box::new(b_n_internal),
        right: Box::new(o_u_internal),
        bit_vec: BitVec::from([
            true, true, false, false, true, true, false, true, true, true, true, false, false,
            true, false, false, true, false, true, true, false, false, true, true, false, true,
            true, false, true, false,
        ]),
    }
}

#[test]
fn new() {
    let data = "tobeornottobethatisthequestion".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    let expected_root = build_test_wt();
    let expected_alphabet = data
        .chars()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let expected_len = 30;

    assert_eq!(wt.root, expected_root);
    assert_eq!(wt.alphabet, expected_alphabet);
    assert_eq!(wt.len, expected_len);
}

#[test]
fn new_when_single_char_alphabet() {
    let data = "aaa".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    let expected_root = WaveletTreeNode::<BitVec>::Leaf { len: 3 };
    let expected_alphabet = vec!['a'];
    let expected_len = 3;

    assert_eq!(wt.root, expected_root);
    assert_eq!(wt.alphabet, expected_alphabet);
    assert_eq!(wt.len, expected_len);
}

#[test]
#[should_panic]
fn new_when_empty_string() {
    let data = "".to_string();
    let spec = BitVec::spec();
    WaveletTree::new(&data, &spec);
}

#[test]
fn access() {
    let data = "tobeornottobethatisthequestion".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    for (i, char) in data.chars().enumerate() {
        assert_eq!(wt.access(i).unwrap(), char);
    }
    assert_eq!(wt.access(data.len()), None);
}

#[test]
fn rank() {
    let data = "tobeornottobethatisthequestion".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    assert_eq!(wt.rank('t', 0).unwrap(), 0);
    assert_eq!(wt.rank('t', 1).unwrap(), 1);
    assert_eq!(wt.rank('t', 2).unwrap(), 1);
    assert_eq!(wt.rank('t', 8).unwrap(), 1);
    assert_eq!(wt.rank('t', 9).unwrap(), 2);

    assert_eq!(wt.rank('o', 0).unwrap(), 0);
    assert_eq!(wt.rank('o', 1).unwrap(), 0);
    assert_eq!(wt.rank('o', 2).unwrap(), 1);
    assert_eq!(wt.rank('o', 30).unwrap(), 5);

    assert_eq!(wt.rank('i', 18).unwrap(), 1);
}

#[test]
fn rank_when_single_char_alphabet() {
    let data = "aaa".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    assert_eq!(wt.rank('a', 0).unwrap(), 0);
    assert_eq!(wt.rank('a', 1).unwrap(), 1);
    assert_eq!(wt.rank('a', 2).unwrap(), 2);
    assert_eq!(wt.rank('a', 3).unwrap(), 3);
}

#[test]
fn rank_when_char_not_in_alphabet() {
    let data = "text".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    assert!(wt.rank('a', 0).is_none());
}

#[test]
fn select() {
    let data = "tobeornottobethatisthequestion".to_string();
    let spec = DenseSamplingRank::spec(4);
    let wt = WaveletTree::new(&data, &spec);

    assert_eq!(wt.select('t', 0).unwrap(), 0);
    assert_eq!(wt.select('t', 1).unwrap(), 1);
    assert_eq!(wt.select('t', 2).unwrap(), 9);
    assert_eq!(wt.select('t', 3).unwrap(), 10);
    assert_eq!(wt.select('t', 4).unwrap(), 14);

    assert_eq!(wt.select('o', 1).unwrap(), 2);
    assert_eq!(wt.select('o', 2).unwrap(), 5);
    assert_eq!(wt.select('o', 3).unwrap(), 8);

    assert_eq!(wt.select('i', 1).unwrap(), 18);
    assert_eq!(wt.select('i', 2).unwrap(), 28);
}

#[test]
fn select_when_single_char_alphabet() {
    let data = "aaa".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    assert_eq!(wt.select('a', 0).unwrap(), 0);
    assert_eq!(wt.select('a', 1).unwrap(), 1);
    assert_eq!(wt.select('a', 2).unwrap(), 2);
    assert_eq!(wt.select('a', 3).unwrap(), 3);
}

#[test]
fn select_when_char_not_in_alphabet() {
    let data = "text".to_string();
    let spec = BitVec::spec();
    let wt = WaveletTree::new(&data, &spec);

    assert!(wt.select('a', 0).is_none());
}

fn random_utf8_text(n: usize) -> String {
    (0..n).map(|_| rand::random::<char>()).collect::<String>()
}

macro_rules! test_for_spec {
    ($($spec:tt),*) => {
        $(
        paste::paste! {
            #[allow(non_snake_case)]
            mod [<when_spec_is_ $spec>] {
                use super::*;
                const SIZE: usize = 10_000;

                #[test]
                fn access_with_random_text() {
                    let text = random_utf8_text(SIZE);
                    let chars = text.chars().collect::<Vec<_>>();
                    let wt = WaveletTree::new(&text, &$spec);
                    for i in 0..SIZE {
                        assert_eq!(wt.access(i).unwrap(), chars[i]);
                    }
                    assert_eq!(wt.access(SIZE), None);
                }

                #[test]
                fn rank_with_random_text() {
                    let text = random_utf8_text(SIZE);
                    let chars = text.chars().collect::<Vec<_>>();
                    let wt = WaveletTree::new(&text, &$spec);
                    let mut acc_ranks = HashMap::<char, usize>::new();
                    let alphabet = wt.alphabet();

                    for c in alphabet {
                        assert_eq!(wt.rank(*c, 0).unwrap(), 0);
                    }

                    for i in 0..SIZE {
                        let c = chars[i];
                        let rank = wt.rank(c, i).unwrap();
                        let acc_rank = acc_ranks.entry(c).or_insert(0);
                        assert_eq!(rank, *acc_rank);
                        *acc_rank += 1;
                    }
                    assert_eq!(wt.rank(chars[0], SIZE+1), None);

                    for c in chars {
                        assert_eq!(acc_ranks[&c],wt.rank(c, SIZE).unwrap());
                    }
                }

                #[test]
                fn select_with_random_text(){
                    let text = random_utf8_text(SIZE);
                    let chars = text.chars().collect::<Vec<_>>();
                    let wt = WaveletTree::new(&text, &$spec);
                    let mut acc_selects = HashMap::<char, usize>::new();
                    let alphabet = wt.alphabet();

                    for c in alphabet {
                        assert_eq!(wt.select(*c, 0).unwrap(), 0);
                    }

                    for i in 0..SIZE {
                        let c = chars[i];
                        let acc_select = acc_selects.entry(c).or_insert(0);
                        *acc_select += 1;
                        let select = wt.select(c, *acc_select).unwrap();
                        assert_eq!(select, i+1);
                    }
                    assert_eq!(wt.select(chars[0], SIZE+1), None);
                }
            }
        }
        )*
    };
}

const BV_SPEC: BitVecSpec = BitVec::spec();
const SPARSE_SAMPLING_RANK_4_SPEC: SparseSamplingRankSpec = SparseSamplingRank::spec(4);
const SPARSE_SAMPLING_RANK_8_SPEC: SparseSamplingRankSpec = SparseSamplingRank::spec(8);
const SPARSE_SAMPLING_RANK_16_SPEC: SparseSamplingRankSpec = SparseSamplingRank::spec(16);
const SPARSE_SAMPLING_RANK_20_SPEC: SparseSamplingRankSpec = SparseSamplingRank::spec(20);
const DENSE_SAMPLING_RANK_4_SPEC: DenseSamplingRankSpec = DenseSamplingRank::spec(4);
const DENSE_SAMPLING_RANK_8_SPEC: DenseSamplingRankSpec = DenseSamplingRank::spec(8);
const DENSE_SAMPLING_RANK_16_SPEC: DenseSamplingRankSpec = DenseSamplingRank::spec(16);
const DENSE_SAMPLING_RANK_20_SPEC: DenseSamplingRankSpec = DenseSamplingRank::spec(20);
const RRR_VEC_3_4_SPEC: RRRBitVecSpec = RRRBitVec::spec(3, 4);
const RRR_VEC_7_4_SPEC: RRRBitVecSpec = RRRBitVec::spec(7, 4);
const RRR_VEC_15_4_SPEC: RRRBitVecSpec = RRRBitVec::spec(15, 4);
const RRR_VEC_31_4_SPEC: RRRBitVecSpec = RRRBitVec::spec(31, 4);
const RRR_VEC_63_4_SPEC: RRRBitVecSpec = RRRBitVec::spec(63, 4);
test_for_spec!(
    BV_SPEC,
    SPARSE_SAMPLING_RANK_4_SPEC,
    SPARSE_SAMPLING_RANK_8_SPEC,
    SPARSE_SAMPLING_RANK_16_SPEC,
    SPARSE_SAMPLING_RANK_20_SPEC,
    DENSE_SAMPLING_RANK_4_SPEC,
    DENSE_SAMPLING_RANK_8_SPEC,
    DENSE_SAMPLING_RANK_16_SPEC,
    DENSE_SAMPLING_RANK_20_SPEC,
    RRR_VEC_3_4_SPEC,
    RRR_VEC_7_4_SPEC,
    RRR_VEC_15_4_SPEC,
    RRR_VEC_31_4_SPEC,
    RRR_VEC_63_4_SPEC
);
