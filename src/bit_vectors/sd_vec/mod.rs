use std::marker::PhantomData;

use crate::getbits;
use crate::int_vectors::CompactIntVec;
use crate::profiling::HeapSize;

use super::rank_select::{Rank, Select};
use super::BitVec;
use crate::Build;

#[derive(Debug)]
pub struct SDVec<T>
where
    T: Select,
{
    r: usize,
    m: usize,
    select_structure: T,
    lower_bits: CompactIntVec,
}

impl<T> SDVec<T>
where
    T: Select,
{
    pub fn new<B: Build<BitVec, T>>(data: BitVec, spec: &B) -> Self {
        let m = data.rank(data.len()).unwrap();
        let n = data.len();

        let r = (n / m).ilog2();

        let mut higher_bits = BitVec::from_value(false, m + (n >> r));

        let r = r as usize;
        let mut lower_bits = CompactIntVec::with_capacity(r, m);

        let mut j = 0;

        for i in 0..n {
            // TODO: optimize this reading the words in raw_data, and shifting right. Stop reading at n-1 so
            // we don't read past the end of the bitvec. the data.read in a loop is very slow.

            if data.read(i) {
                // push i mod 2^r to L
                lower_bits.push(getbits!(i, r, 0));
                // i >> r indicates how many blocks of size 2^r we have before. we add j
                // because of the number of 1s set in the unary code.
                higher_bits.set((i >> r) + j, true);
                // sum j after, because we index bitvecs starting from 0
                j += 1;
            }
        }

        let select_structure = spec.build(higher_bits);
        Self {
            r,
            m,
            select_structure,
            lower_bits,
        }
    }
}

impl<T> Select for SDVec<T>
where
    T: Select,
{
    fn select(&self, rank: usize) -> Option<usize> {
        if rank == 0 {
            return Some(0);
        }

        if rank > self.m {
            return None;
        }

        let higher = (self.select_structure.select(rank).unwrap() - rank) << self.r;
        let lower = self.lower_bits.get(rank - 1).unwrap();

        // this could be Some(higher | lower)?
        // +1 because higher | lower gives us the exact location of the bit, but for select
        // we want to return the next
        Some((higher | lower) + 1)
    }

    fn select0(&self, _rank0: usize) -> Option<usize> {
        todo!()
    }
}

impl<T> HeapSize for SDVec<T>
where
    T: Select + HeapSize,
{
    fn heap_size_in_bits(&self) -> usize {
        self.select_structure.heap_size_in_bits() + self.lower_bits.heap_size_in_bits()
    }
}

pub struct SDVecSpec<B, T>
where
    B: Build<BitVec, T>,
    T: Select,
{
    spec: B,
    phantom: PhantomData<T>,
}

impl<B, T> SDVecSpec<B, T>
where
    B: Build<BitVec, T>,
    T: Select,
{
    pub const fn new(spec: B) -> Self {
        Self {
            spec,
            phantom: PhantomData,
        }
    }
}

impl<T> SDVec<T>
where
    T: Select,
{
    pub const fn spec<B: Build<BitVec, T>>(spec: B) -> SDVecSpec<B, T> {
        SDVecSpec::new(spec)
    }
}

impl<B, T> Build<BitVec, SDVec<T>> for SDVecSpec<B, T>
where
    B: Build<BitVec, T>,
    T: Select,
{
    fn build(&self, data: BitVec) -> SDVec<T> {
        SDVec::new(data, &self.spec)
    }
}

#[cfg(test)]
mod tests {
    use crate::bit_vectors::rank_select::sparse_sampling_rank::SparseSamplingRankSpec;
    use crate::bit_vectors::rank_select::RankStructure;
    use crate::util::BitsRequired;

    use super::*;
    #[test]
    fn new() {
        let spec = super::super::rank_select::SparseSamplingRank::spec(4);
        let bv = BitVec::from([
            0b00101010u8,
            0b10001010,
            0b00000100,
            0b00000000,
            0b01001100,
            0b00001011,
            0b01000100,
            0b00010000,
        ]);
        let sd_vec = SDVec::new(bv, &spec);

        println!("M: {}", sd_vec.m);
        println!("R: {}", sd_vec.r);
        println!("L: {}", sd_vec.lower_bits);
        println!("H: {}", sd_vec.select_structure.data());

        dbg!(sd_vec.select(0));
        dbg!(sd_vec.select(1).unwrap() + 1);
        dbg!(sd_vec.select(2).unwrap() + 1);
        dbg!(sd_vec.select(3).unwrap() + 1);
        dbg!(sd_vec.select(4).unwrap() + 1);
        dbg!(sd_vec.select(5).unwrap() + 1);
        dbg!(sd_vec.select(6).unwrap() + 1);
        dbg!(sd_vec.select(7).unwrap() + 1);
        dbg!(sd_vec.select(8).unwrap() + 1);
        dbg!(sd_vec.select(9).unwrap() + 1);
        dbg!(sd_vec.select(10).unwrap() + 1);
        dbg!(sd_vec.select(11).unwrap() + 1);
        dbg!(sd_vec.select(12).unwrap() + 1);
        dbg!(sd_vec.select(13).unwrap() + 1);
        dbg!(sd_vec.select(14).unwrap() + 1);
        dbg!(sd_vec.select(15).unwrap() + 1);
        dbg!(sd_vec.select(16).unwrap() + 1);
        dbg!(sd_vec.select(17));
    }
    use crate::bit_vectors::rank_select::tests_utils::test_select_for;
    use crate::bit_vectors::rank_select::SparseSamplingRank;
    test_select_for!(
        SDVec<RankStructure<BitVec, SparseSamplingRank>>,
        SparseSamplingRank::spec(4)
    );
}
