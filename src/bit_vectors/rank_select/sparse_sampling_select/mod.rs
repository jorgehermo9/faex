use crate::bit_vectors::rank_select::Rank;
use crate::bit_vectors::BitVec;
use crate::int_vectors::CompactIntVec;
use crate::profiling::HeapSize;
use crate::util::{ceil_div, BitsRequired};

use super::{SelectStructure, SelectSupport, SparseSamplingRank};
use crate::Build;
pub struct SparseSamplingSelect {
    s: usize,
    total_rank: usize,
    select_samples: CompactIntVec,
    sparse_sample_rank: SparseSamplingRank,
}

impl SparseSamplingSelect {
    pub fn new(data: &BitVec, s: usize, k: usize) -> Self {
        assert!(s > 0, "s must be greater than 0");
        assert!(k > 0, "k must be greater than 0");

        let n = data.len();

        let bits_required = n.bits_required() as usize;
        let mut select_samples = CompactIntVec::new(bits_required);

        let mut r = 0;
        for i in 0..n {
            if data.read(i) {
                if r % s == 0 {
                    select_samples.push(i);
                }
                r += 1;
            }
        }

        let spec = SparseSamplingRank::spec(k);
        let sparse_sample_rank = spec.build(data);

        select_samples.push(n);

        Self {
            s,
            total_rank: r,
            select_samples,
            sparse_sample_rank,
        }
    }

    pub fn s(&self) -> usize {
        self.s
    }
}

impl SelectSupport<BitVec> for SparseSamplingSelect {
    unsafe fn select(&self, data: &BitVec, rank: usize) -> Option<usize> {
        if rank == 0 {
            return Some(0);
        }

        if rank > self.total_rank {
            return None;
        }

        let p = (rank - 1) / self.s;
        let left = self.select_samples.get(p)? / self.sparse_sample_rank.superblock_size();
        let right = self.select_samples.get(p + 1)? / self.sparse_sample_rank.superblock_size();

        self.sparse_sample_rank
            .select_with_hints(data, rank, left, right)
    }

    unsafe fn select0(&self, data: &BitVec, rank0: usize) -> Option<usize> {
        todo!()
    }
}

pub struct SparseSamplingSelectSpec {
    s: usize,
    k: usize,
}

impl SparseSamplingSelectSpec {
    pub const fn new(s: usize, k: usize) -> Self {
        Self { s, k }
    }
}

impl SparseSamplingSelect {
    pub const fn spec(s: usize, k: usize) -> SparseSamplingSelectSpec {
        SparseSamplingSelectSpec::new(s, k)
    }
}

impl Build<BitVec, SelectStructure<BitVec, SparseSamplingSelect>> for SparseSamplingSelectSpec {
    fn build(&self, data: BitVec) -> SelectStructure<BitVec, SparseSamplingSelect> {
        let sparse_sampling_select = SparseSamplingSelect::new(&data, self.s, self.k);
        unsafe { SelectStructure::new(data, sparse_sampling_select) }
    }
}

impl Build<&BitVec, SparseSamplingSelect> for SparseSamplingSelectSpec {
    fn build(&self, data: &BitVec) -> SparseSamplingSelect {
        SparseSamplingSelect::new(data, self.s, self.k)
    }
}

impl HeapSize for SparseSamplingSelect {
    fn heap_size_in_bits(&self) -> usize {
        self.select_samples.heap_size_in_bits() + self.sparse_sample_rank.heap_size_in_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bit_vectors::rank_select::tests_utils::test_select_for;
    #[test]
    fn new() {
        use super::*;
        let bv = BitVec::from([0b10101010usize; 2]);
        let ss = SparseSamplingSelect::new(&bv, 4, 1);
        assert_eq!(ss.s(), 4);
        println!("{}", ss.select_samples);
        dbg!(unsafe { ss.select(&bv, 9) });
    }
    test_select_for!(SparseSamplingSelect, 4, 4);
}
