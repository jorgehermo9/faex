//! Rank/Select data structure with a 1-level directory of superblocks.
//! Each superblock has size `s = k * w`, where `w` is the word size of the
//! machine, so `w` matches with the size of the type `usize`. Each superblock
//! stores Rank(floor(i/s)) for every index `i`.

use super::{RankStructure, RankSupport, SelectSupport};
use crate::Build;
use crate::{bit_vectors::BitVec, profiling::HeapSize};

// TODO: improve superblocks by taking into account that the maximum value
// it could hold is the number of 1s in the bitvector, using CompactIntVec,
// as this structure is immutable.
#[derive(Debug)]
pub struct SparseSamplingRank {
    superblocks: Vec<usize>,
    superblock_size: usize,
    total_rank: usize,
    k: usize,
}

impl SparseSamplingRank {
    #[inline]
    pub fn new(data: &BitVec, k: usize) -> Self {
        assert!(k > 0, "k must be greater than 0");

        let superblock_size = k * BitVec::CONTAINER_WIDTH;
        let num_superblocks = data.len() / superblock_size;
        let mut superblocks = Vec::with_capacity(num_superblocks + 1);

        let mut rank = 0;
        // first superblock is always 0
        superblocks.push(rank);

        let raw_data = data.raw_data();
        for i in 0..num_superblocks {
            for j in i * k..(i + 1) * k {
                rank += unsafe { raw_data.get_unchecked(j) }.count_ones() as usize;
            }
            superblocks.push(rank);
        }

        let mut unsampled_rank = 0;
        for i in num_superblocks * k..raw_data.len() {
            unsampled_rank += unsafe { raw_data.get_unchecked(i) }.count_ones() as usize;
        }

        // For technical reasons, we need to add the total rank as the last superblock.
        // This allows us to access the total rank (altought we store it in the struct),
        // but also it allows us to perform binary search correctly, because otherwise, we
        // may perform a sequential search within two superblocks. In this way,
        // the binary search right position will be always >= the rank we are looking for.
        // but if we didn't do this, it could be that the last position could be < the rank we are looking for,
        // and wont be selected as the greatest of the lessers.
        // It is not necessary to add the total rank if all the blocks are fully sampled by a superblock, since the
        // last superblock will be the total rank.
        if unsampled_rank != 0 {
            rank += unsampled_rank;
            superblocks.push(rank);
        }

        Self {
            superblocks,
            superblock_size,
            k,
            total_rank: rank,
        }
    }

    pub fn superblocks(&self) -> &[usize] {
        &self.superblocks
    }

    pub fn superblock_size(&self) -> usize {
        self.superblock_size
    }

    pub fn k(&self) -> usize {
        self.k
    }

    #[inline]
    pub(crate) unsafe fn select_with_hints(
        &self,
        data: &BitVec,
        rank: usize,
        left: usize,
        right: usize,
    ) -> Option<usize> {
        let mut left = left;
        let mut right = right;

        while right - left > 1 {
            let mid = (left + right) / 2;
            let mid_rank = *self.superblocks.get_unchecked(mid);
            if mid_rank < rank {
                left = mid;
            } else {
                right = mid;
            }
        }

        // search for the block that contains the rank.
        // We know for sure that in left position is the superblock value that is the greatest
        // of the ranks <= rank, as the last position is always greater or equal than the rank,
        // as the total rank is the last superblock value.
        let mut local_rank = *self.superblocks.get_unchecked(left);
        let mut block_index = left * self.k;
        let raw_data = data.raw_data();
        let mut block_rank = raw_data.get_unchecked(block_index).count_ones() as usize;

        // Stop where at the next block step, the rank is greater than the rank we are looking for
        while local_rank + block_rank < rank {
            local_rank += block_rank;
            block_index += 1;
            block_rank = raw_data.get_unchecked(block_index).count_ones() as usize;
        }

        // at this point, we are exactly that the block that contains the rank is `block_index`
        let mut block = *raw_data.get_unchecked(block_index);

        // select the bit in the block
        let mut bit_index = 0;
        while local_rank < rank {
            if block & 0b1 == 0b1 {
                local_rank += 1;
            }
            block >>= 1;
            bit_index += 1;
        }

        Some(block_index * BitVec::CONTAINER_WIDTH + bit_index)
    }

    #[inline]
    pub(crate) unsafe fn select0_with_hints(
        &self,
        data: &BitVec,
        rank0: usize,
        left: usize,
        right: usize,
    ) -> Option<usize> {
        let mut left = left;
        let mut right = right;

        while right - left > 1 {
            let mid = (left + right) / 2;
            let bits_before_mid = mid * self.superblock_size;
            let mid_rank0 = bits_before_mid - *self.superblocks.get_unchecked(mid);
            if mid_rank0 < rank0 {
                left = mid;
            } else {
                right = mid;
            }
        }

        // search for the block that contains the rank
        let bits_before_left = left * self.superblock_size;
        let mut local_rank0 = bits_before_left - *self.superblocks.get_unchecked(left);
        let mut block_index = left * self.k;
        let raw_data = data.raw_data();
        // We dont have to check if the block_index is at last and we are counting bits that are
        // not filled (i.e, data.len() is not multiple of container width),since we already check the max
        // number of 0s that the vector have, and we know that we always search for a valid rank
        let mut block_rank0 = raw_data.get_unchecked(block_index).count_zeros() as usize;

        // Stop where at the next block step, the rank is greater than the rank we are looking for
        while local_rank0 + block_rank0 < rank0 {
            local_rank0 += block_rank0;
            block_index += 1;
            block_rank0 = raw_data.get_unchecked(block_index).count_zeros() as usize;
        }

        // at this point, we are exactly that the block that contains the rank is `block_index`
        let mut block = *raw_data.get_unchecked(block_index);

        // select the bit in the block
        let mut bit_index = 0;
        while local_rank0 < rank0 {
            if block & 0b1 == 0b0 {
                local_rank0 += 1;
            }
            block >>= 1;
            bit_index += 1;
        }

        Some(block_index * BitVec::CONTAINER_WIDTH + bit_index)
    }
}

impl HeapSize for SparseSamplingRank {
    #[inline]
    fn heap_size_in_bits(&self) -> usize {
        self.superblocks.heap_size_in_bits()
    }
}

impl RankSupport<BitVec> for SparseSamplingRank {
    #[inline]
    unsafe fn rank(&self, data: &BitVec, index: usize) -> Option<usize> {
        // By definition, rank(0) = 0
        if index == 0 {
            return Some(0);
        }

        if index > data.len() {
            return None;
        }

        let is = index / self.superblock_size;
        let iw = index / BitVec::CONTAINER_WIDTH;

        let mut rank = self.superblocks[is];

        let raw_data = data.raw_data();
        for i in is * self.k..iw {
            // If the data is not the one for which this rank was computed, then
            // we may end in undefined behavior...
            rank += raw_data.get_unchecked(i).count_ones() as usize;
        }

        let block_offset = index % BitVec::CONTAINER_WIDTH;
        let last_block = raw_data.get(iw).copied().unwrap_or(0);
        // getbits!(last_block, block_offset, 0) optimized, we know that block_offset is always <64 so this does not overflow
        let last_block_target = last_block & ((1 << block_offset) - 1);
        let last_block_rank = last_block_target.count_ones() as usize;

        Some(rank + last_block_rank)
    }
}

impl SelectSupport<BitVec> for SparseSamplingRank {
    // TODO: extract select in hinted_select private function of sparse sampling rank,
    // so it could be used from here with range (0, len-1) and in
    // sparse select with range i1,i2
    #[inline]
    unsafe fn select(&self, data: &BitVec, rank: usize) -> Option<usize> {
        // // By definition, select(0) = 0
        if rank == 0 {
            return Some(0);
        }

        if rank > self.total_rank {
            return None;
        }

        let left = 0;
        let right = self.superblocks.len() - 1;

        self.select_with_hints(data, rank, left, right)
    }

    #[inline]
    unsafe fn select0(&self, data: &BitVec, rank0: usize) -> Option<usize> {
        // // By definition, select0(0) = 0
        if rank0 == 0 {
            return Some(0);
        }
        let total_rank0 = data.len() - self.total_rank;
        if rank0 > total_rank0 {
            return None;
        }

        let left = 0;
        let right = self.superblocks.len() - 1;

        self.select0_with_hints(data, rank0, left, right)
    }
}

pub struct SparseSamplingRankSpec {
    pub k: usize,
}

impl SparseSamplingRankSpec {
    #[inline]
    pub const fn new(k: usize) -> Self {
        Self { k }
    }
}

impl SparseSamplingRank {
    #[inline]
    pub const fn spec(k: usize) -> SparseSamplingRankSpec {
        SparseSamplingRankSpec::new(k)
    }
}

impl Build<BitVec, RankStructure<BitVec, SparseSamplingRank>> for SparseSamplingRankSpec {
    #[inline]
    fn build(&self, data: BitVec) -> RankStructure<BitVec, SparseSamplingRank> {
        let sparse_sampling = SparseSamplingRank::new(&data, self.k);
        unsafe { RankStructure::new(data, sparse_sampling) }
    }
}
impl Build<&BitVec, SparseSamplingRank> for SparseSamplingRankSpec {
    #[inline]
    fn build(&self, data: &BitVec) -> SparseSamplingRank {
        SparseSamplingRank::new(data, self.k)
    }
}

#[cfg(test)]
mod tests;
