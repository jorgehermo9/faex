use super::{RankStructure, RankSupport, SelectSupport};
use crate::Build;
use crate::{
    bit_vectors::BitVec, int_vectors::compact_int_vec::CompactIntVec, profiling::HeapSize,
    util::BitsRequired,
};
use serde::{Deserialize, Serialize};
use std::cmp::min;

///
#[derive(Debug, Serialize, Deserialize)]
pub struct DenseSamplingRank {
    superblocks: Vec<usize>,
    superblock_size: usize,
    blocks: CompactIntVec,
    k: usize,
    total_rank: usize,
}

impl DenseSamplingRank {
    #[inline]
    pub fn new(data: &BitVec, k: usize) -> Self {
        assert!(k > 0, "k must be greater than 0");

        let superblock_size = k * BitVec::CONTAINER_WIDTH;
        let num_superblocks = data.len() / superblock_size;
        let mut superblocks = Vec::with_capacity(num_superblocks + 1);

        // This could be superblock_size - block_size, but in practice it is enough with -1
        let max_rank_offset_value = (k - 1) * BitVec::CONTAINER_WIDTH;
        let rank_offset_int_width = max_rank_offset_value.bits_required() as usize;

        let num_blocks = data.len() / BitVec::CONTAINER_WIDTH;
        let mut blocks = CompactIntVec::with_capacity(rank_offset_int_width, num_blocks + 1);

        let mut rank = 0;
        let mut rank_offset = 0;
        // Push initial 0 values for convenience
        superblocks.push(rank);
        blocks.push(rank_offset);

        let raw_data = data.raw_data();
        for i in 0..num_superblocks {
            for j in i * k..(i + 1) * k - 1 {
                rank_offset += unsafe { raw_data.get_unchecked(j) }.count_ones() as usize;
                blocks.push(rank_offset);
            }
            let last_block_rank =
                unsafe { raw_data.get_unchecked((i + 1) * k - 1) }.count_ones() as usize;
            rank_offset += last_block_rank;
            rank += rank_offset;
            rank_offset = 0;

            blocks.push(rank_offset);
            superblocks.push(rank);
        }

        let mut unsampled_superblock_rank = 0;
        // There may be more blocks after the last superblock
        for i in num_superblocks * k..num_blocks {
            unsampled_superblock_rank += unsafe { raw_data.get_unchecked(i) }.count_ones() as usize;
            blocks.push(unsampled_superblock_rank);
        }

        // If the last block is not full, we need to add the remaining 1s to the unsampled rank,
        // but we do not push a value for that block, since it is not fully block-sampled and we
        // cannot do the same trick for the binary search as we do for the superblocks.
        if num_blocks != raw_data.len() {
            unsampled_superblock_rank += raw_data.last().unwrap().count_ones() as usize;
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
        if unsampled_superblock_rank != 0 {
            rank += unsampled_superblock_rank;
            superblocks.push(rank);
        }

        Self {
            superblocks,
            superblock_size,
            blocks,
            k,
            total_rank: rank,
        }
    }

    #[inline]
    pub fn superblocks(&self) -> &[usize] {
        &self.superblocks
    }

    #[inline]
    pub fn superblock_size(&self) -> usize {
        self.superblock_size
    }

    #[inline]
    pub fn blocks(&self) -> &CompactIntVec {
        &self.blocks
    }

    #[inline]
    pub fn k(&self) -> usize {
        self.k
    }
}
impl HeapSize for DenseSamplingRank {
    #[inline]
    fn heap_size_in_bits(&self) -> usize {
        // Do not rely on underlying data structure's heap_size_in_bits,
        // since we want to count the heap size of the rank structure without
        // static overhead (i.e first block/superblock that is pushed as a 0 value for convenience)
        self.superblocks.heap_size_in_bits() + self.blocks.heap_size_in_bits()
    }
}

impl RankSupport<BitVec> for DenseSamplingRank {
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

        let rank = self.superblocks.get_unchecked(is) + self.blocks.get_unchecked(iw);
        let block_offset = index % BitVec::CONTAINER_WIDTH;
        let last_block = data.raw_data().get(iw).copied().unwrap_or(0);

        // getbits!(last_block, block_offset, 0) optimized, we know that block_offset is always <64 so this does not overflow
        let last_block_target = last_block & ((1 << block_offset) - 1);
        let last_block_rank = last_block_target.count_ones() as usize;

        Some(rank + last_block_rank)
    }
}

impl SelectSupport<BitVec> for DenseSamplingRank {
    #[inline]
    unsafe fn select(&self, data: &BitVec, rank: usize) -> Option<usize> {
        // By definition, select(0) = 0
        if rank == 0 {
            return Some(0);
        }

        if rank > self.total_rank {
            return None;
        }

        let mut left_superblock = 0;
        let mut right_superblock = self.superblocks.len() - 1;

        while right_superblock - left_superblock > 1 {
            let mid_superblock = (left_superblock + right_superblock) / 2;
            let mid_rank = *self.superblocks.get_unchecked(mid_superblock);
            if mid_rank < rank {
                left_superblock = mid_superblock;
            } else {
                right_superblock = mid_superblock;
            }
        }

        // search for the block that contains the rank.
        // We know for sure that in left position is the superblock value that is the greatest
        // of the ranks <= rank, as the last position is always greater or equal than the rank,
        // as the total rank is the last superblock value.
        let superblock_rank = *self.superblocks.get_unchecked(left_superblock);
        let remaining_rank = rank - superblock_rank;
        let raw_data = data.raw_data();
        let mut left_block_index = left_superblock * self.k;

        let mut right_block_index = min(left_block_index + self.k - 1, raw_data.len() - 1);
        while right_block_index - left_block_index > 1 {
            let mid = (left_block_index + right_block_index) / 2;
            let mid_rank = self.blocks.get_unchecked(mid);
            if mid_rank < remaining_rank {
                left_block_index = mid;
            } else {
                right_block_index = mid;
            }
        }

        // We cannot do the same trick as in the previous binary search, so we have to check
        // if the rank at the right block is less than the remaining rank, and if so, we select
        // the right block, otherwise, we select the left block. This happens for example
        // when searching by number 5 in the following blockvector: `1 2 3 4`, left would
        // point to 3 and right to 4, but our target_block_index must be the one targeted by `4`
        let right_rank = self.blocks.get_unchecked(right_block_index);
        // check whether the right index contains the greatest of the lessers (we do not have the binary
        // search bounds trick here)
        let target_block_index = if right_rank < remaining_rank {
            right_block_index
        } else {
            left_block_index
        };

        let mut local_rank = self.blocks.get_unchecked(target_block_index);

        // at this point, we are exactly that the block that contains the rank is `target_block_index`
        let mut block = *raw_data.get_unchecked(target_block_index);

        // select the bit in the block
        let mut bit_index = 0;
        while local_rank < remaining_rank {
            if block & 0b1 == 0b1 {
                local_rank += 1;
            }
            block >>= 1;
            bit_index += 1;
        }

        Some(target_block_index * BitVec::CONTAINER_WIDTH + bit_index)
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

        let mut left = 0;
        let mut right = self.superblocks.len() - 1;

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

        // search for the block that contains the rank.
        // We know for sure that in left position is the superblock value that is the greatest
        // of the ranks <= rank, as the last position is always greater or equal than the rank,
        // as the total rank is the last superblock value.
        // search for the block that contains the rank
        let bits_before_left = left * self.superblock_size;

        let superblock_rank0 = bits_before_left - *self.superblocks.get_unchecked(left);
        let remaining_rank0 = rank0 - superblock_rank0;
        let raw_data = data.raw_data();

        let first_block_index = left * self.k;
        let mut left_block_index = first_block_index;

        let mut right_block_index = min(left_block_index + self.k - 1, raw_data.len() - 1);
        while right_block_index - left_block_index > 1 {
            let mid = (left_block_index + right_block_index) / 2;
            let bits_before_mid = (mid - first_block_index) * BitVec::CONTAINER_WIDTH;
            let mid_rank0 = bits_before_mid - self.blocks.get_unchecked(mid);
            if mid_rank0 < remaining_rank0 {
                left_block_index = mid;
            } else {
                right_block_index = mid;
            }
        }

        // We cannot do the same trick as in the previous binary search, so we have to check
        // if the rank at the right block is less than the remaining rank, and if so, we select
        // the right block, otherwise, we select the left block. This happens for example
        // when searching by number 5 in the following blockvector: `1 2 3 4`, left would
        // point to 3 and right to 4, but our target_block_index must be the one targeted by `4`
        let bits_before_right = (right_block_index - first_block_index) * BitVec::CONTAINER_WIDTH;
        let right_rank0 = bits_before_right - self.blocks.get_unchecked(right_block_index);
        // check whether the right index contains the greatest of the lessers (we do not have the binary
        // search bounds trick here)
        let target_block_index = if right_rank0 < remaining_rank0 {
            right_block_index
        } else {
            left_block_index
        };

        let bits_before_target = (target_block_index - first_block_index) * BitVec::CONTAINER_WIDTH;
        let mut local_rank0 = bits_before_target - self.blocks.get_unchecked(target_block_index);

        // at this point, we are exactly that the block that contains the rank is `target_block_index`
        let mut block = *raw_data.get_unchecked(target_block_index);

        // select the bit in the block
        let mut bit_index = 0;
        while local_rank0 < remaining_rank0 {
            if block & 0b1 == 0b0 {
                local_rank0 += 1;
            }
            block >>= 1;
            bit_index += 1;
        }

        Some(target_block_index * BitVec::CONTAINER_WIDTH + bit_index)
    }
}

pub struct DenseSamplingRankSpec {
    pub k: usize,
}

impl DenseSamplingRankSpec {
    #[inline]
    pub const fn new(k: usize) -> Self {
        Self { k }
    }
}

impl DenseSamplingRank {
    #[inline]
    pub const fn spec(k: usize) -> DenseSamplingRankSpec {
        DenseSamplingRankSpec::new(k)
    }
}

impl Build<BitVec, RankStructure<BitVec, DenseSamplingRank>> for DenseSamplingRankSpec {
    #[inline]
    fn build(&self, data: BitVec) -> RankStructure<BitVec, DenseSamplingRank> {
        let sparse_sampling = DenseSamplingRank::new(&data, self.k);
        unsafe { RankStructure::new(data, sparse_sampling) }
    }
}

impl Build<&BitVec, DenseSamplingRank> for DenseSamplingRankSpec {
    #[inline]
    fn build(&self, data: &BitVec) -> DenseSamplingRank {
        DenseSamplingRank::new(data, self.k)
    }
}

#[cfg(test)]
mod tests;
