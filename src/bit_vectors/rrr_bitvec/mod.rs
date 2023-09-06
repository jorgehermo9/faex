use super::{
    rank_select::{Rank, Select},
    Access, BitVec,
};
use crate::Build;
use crate::{int_vectors::CompactIntVec, profiling::HeapSize, util::BitsRequired};

#[derive(Debug, Clone)]
pub struct RRRBitVec {
    b: usize,
    k: usize,
    classes: CompactIntVec,
    // Use Vec<usize> for lengths instead of a CompactIntVec, since the space overhead
    // is very low and very fast access is required.
    lengths: Vec<usize>,
    offsets: BitVec,
    // TODO: use Vec<usize> for samples? it may be faster, but expensive
    offset_samples: CompactIntVec,
    rank_samples: CompactIntVec,
    // total rank is useful for select operation
    total_rank: usize,
    len: usize,
}

const N: usize = std::mem::size_of::<usize>() * 8;
static BINOMIALS: [[usize; N + 1]; N + 1] = get_binomial_table();

// Precompute binomial table at compile time
const fn get_binomial_table() -> [[usize; N + 1]; N + 1] {
    let mut binomials = [[0; N + 1]; N + 1];

    let mut i = 0;
    // Initialize diagonal
    while i <= N {
        binomials[i][i] = 1;
        i += 1;
    }
    // First row is initialized with 0s

    // Initialize first column
    i = 0;
    while i <= N {
        binomials[i][0] = 1;
        i += 1;
    }

    let mut n = 1;
    let mut k = 1;

    while n <= N {
        while k < n {
            binomials[n][k] = binomials[n - 1][k - 1] + binomials[n - 1][k];
            k += 1;
        }
        n += 1;
        k = 1;
    }

    binomials
}

impl RRRBitVec {
    #[inline]
    // TODO: Array K as precomputed binomials
    fn encode(block: usize, b: usize) -> (usize, usize) {
        let class = block.count_ones() as usize;
        let mut offset = 0;
        let mut current_class = class;
        let mut current_block = block;
        for i in 1..=b {
            // if there are no more 1s or all of the remaining bits are 1s
            if current_class == 0 || current_class > (b - i) {
                break;
            }

            // index bit starting from the right
            if current_block & 0b1 == 0b1 {
                offset += BINOMIALS[b - i][current_class];
                current_class -= 1;
            }
            current_block >>= 1;
        }

        (class, offset)
    }

    // Optimization to decode only len bits
    #[inline]
    fn decode(class: usize, offset: usize, b: usize, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        if class == b {
            // The block is all ones
            return usize::MAX >> (usize::BITS - len as u32);
        } else if class == 0 {
            // The block is all zeros
            return 0;
        }

        let mut block = 0;
        let mut current_class = class;
        let mut current_offset = offset;
        let mut i = 0;
        while current_class > 1 {
            if i >= len {
                return block;
            }
            let num_previous_offsets = BINOMIALS[b - i - 1][current_class];
            if current_offset >= num_previous_offsets {
                // If the bit at ith position (starting from the right) is set,
                // there are b-i-1 choose current_class combinations that
                // have a 0 in that position and precede that offset

                // setbits!(block, 1, i, 1); optimized
                block |= 1 << i;
                current_offset -= num_previous_offsets;
                current_class -= 1;
            }
            i += 1;
        }

        // In case the class is 1, the bit at position b - offset - 1 is set.
        // since there are only b possible combinations.s
        if current_class > 0 {
            let bit_offset = b - current_offset - 1;
            if bit_offset < len {
                block |= 1 << bit_offset;
            }
        }

        block
    }

    #[inline]
    pub fn new(bitvec: BitVec, b: usize, k: usize) -> Self {
        // TODO: allow for greater b values? Now it corresponds
        // to the size of usize (which is also the size of BitVec::CONTAINER_WIDTH)
        assert!(b <= N, "b must be at most {}", N);
        assert!(b > 0, "b must be greater than 0");
        assert!(k > 0, "k must be greater than 0");

        // TODO: use Vec<usize> instead of this compact vec for lenghts? Using compact int vec
        // is useful since we know the max value the vector would hold, but acces
        // may be slower..
        // We know the max value the vector would hold(log log (b choose b/2)) so we can use compact int vec
        // for b =64, if we use Vec<usize>, that would use 4096 bits, but with compact int vec, we use 384 bits

        let len = bitvec.len();

        // let mut lengths = CompactIntVec::with_capacity(compact_vec_width, b + 1);
        let mut lengths = Vec::with_capacity(b + 1);

        lengths.extend((0..=b).map(|c| (BINOMIALS[b][c] - 1).bits_required() as usize));
        let lengths = lengths;

        let blocks = CompactIntVec::from_raw_parts(bitvec, b);
        let mut total_offsets_size = 0;
        let mut total_rank = 0;
        for block in blocks.iter() {
            let class = block.count_ones() as usize;
            total_offsets_size += unsafe { lengths.get_unchecked(class) };
            total_rank += class;
        }

        let classes_width = b.bits_required() as usize;
        let mut classes = CompactIntVec::with_capacity(classes_width, blocks.len());
        let mut offsets = BitVec::with_capacity(total_offsets_size);

        let mut rank_samples =
            CompactIntVec::with_capacity(total_rank.bits_required() as usize, blocks.len() / k + 1);
        let mut offset_samples = CompactIntVec::with_capacity(
            total_offsets_size.bits_required() as usize,
            blocks.len() / k + 1,
        );

        let mut current_rank = 0;
        let mut current_offset_pos = 0;

        for (idx, block) in blocks.iter().enumerate() {
            if idx % k == 0 {
                rank_samples.push(current_rank);
                offset_samples.push(current_offset_pos);
            }
            let (class, offset) = Self::encode(block, b);
            let offset_size = unsafe { lengths.get_unchecked(class) };
            classes.push(class);
            offsets.push_bits(offset, *offset_size);

            current_rank += class;
            current_offset_pos += offset_size;
        }

        // For technical reasons, we need to add the total rank as the last superblock (rank sample).
        // This allows us to access the total rank (altought we store it in the struct),
        // but also it allows us to perform binary search correctly, because otherwise, we
        // may perform a sequential search within two superblocks. In this way,
        // the binary search right position will be always >= the rank we are looking for.
        // but if we didn't do this, it could be that the last position could be < the rank we are looking for,
        // and wont be selected as the greatest of the lessers.
        // It is not necessary to add the total rank if all the blocks are fully sampled by a superblock, since the
        // last superblock will be the total rank.
        // We always push that value, since at the end of the for loop, we always have at least one block not sampled
        // (by the if idx % k == 0 condition being checked before summing the rank)
        rank_samples.push(current_rank);

        // After processing all blocks, we may push a last offset sample.
        if blocks.len() % k == 0 {
            offset_samples.push(current_offset_pos);
        }

        Self {
            b,
            k,
            classes,
            offsets,
            lengths,
            rank_samples,
            offset_samples,
            total_rank: current_rank,
            len,
        }
    }

    #[inline]
    pub fn read(&self, i: usize) -> bool {
        assert!(
            i < self.len(),
            "index out of bounds: the len is {} but the index is {i}",
            self.len()
        );
        unsafe { self.read_unchecked(i) }
    }

    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior
    #[inline]
    pub unsafe fn read_unchecked(&self, i: usize) -> bool {
        let block_index = i / self.b;
        let class = self.classes.get_unchecked(block_index);
        // Very effective optimization :)
        if class == 0 || class == self.b {
            return class > 0;
        }

        let is = block_index / self.k;
        let sampled_pos = self.offset_samples.get_unchecked(is);

        let mut pos = sampled_pos;
        for i in is * self.k..block_index {
            let c = self.classes.get_unchecked(i);
            pos += self.lengths.get_unchecked(c);
        }

        let len = self.lengths.get_unchecked(class);
        let offset = self.offsets.read_bits_unchecked(pos, *len);

        let bit_offset = i % self.b;
        let block = Self::decode(class, offset, self.b, bit_offset + 1);
        // Optimize this as we know we are reading 1 bit
        // getbits!(block, 1, bit_offset)
        block >> bit_offset & 0b1 == 0b1
    }

    #[inline]
    pub fn b(&self) -> usize {
        self.b
    }

    #[inline]
    pub fn k(&self) -> usize {
        self.k
    }

    #[inline]
    pub fn classes(&self) -> &CompactIntVec {
        &self.classes
    }

    #[inline]
    pub fn offsets(&self) -> &BitVec {
        &self.offsets
    }

    #[inline]
    pub fn lengths(&self) -> &[usize] {
        &self.lengths
    }

    #[inline]
    pub fn rank_samples(&self) -> &CompactIntVec {
        &self.rank_samples
    }

    #[inline]
    pub fn offset_samples(&self) -> &CompactIntVec {
        &self.offset_samples
    }

    #[inline]
    pub fn total_rank(&self) -> usize {
        self.total_rank
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl HeapSize for RRRBitVec {
    #[inline]
    fn heap_size_in_bits(&self) -> usize {
        self.classes.heap_size_in_bits()
            + self.offsets.heap_size_in_bits()
            + self.lengths.heap_size_in_bits()
            + self.rank_samples.heap_size_in_bits()
            + self.offset_samples.heap_size_in_bits()
    }
}

impl Rank for RRRBitVec {
    #[inline]
    fn rank(&self, index: usize) -> Option<usize> {
        // By definition, Rank(0) = 0
        if index == 0 {
            return Some(0);
        }

        if index > self.len() {
            return None;
        }

        let block_index = index / self.b;
        let is = block_index / self.k;

        let mut r = unsafe { self.rank_samples.get_unchecked(is) };
        let mut p = unsafe { self.offset_samples.get_unchecked(is) };

        // TODO: optimization that uses next sample difference to detect
        // if all ones or all zeros?

        let iw = index / self.b;
        for i in is * self.k..iw {
            let c = unsafe { self.classes.get_unchecked(i) };
            r += c;
            p += unsafe { self.lengths.get_unchecked(c) };
        }

        let last_block = self.classes.get(iw).map_or(0, |last_class| {
            let block_offset = index % self.b;
            let last_offset = unsafe {
                self.offsets
                    .read_bits_unchecked(p, *self.lengths.get_unchecked(last_class))
            };
            Self::decode(last_class, last_offset, self.b, block_offset)
        });

        let last_block_rank = last_block.count_ones() as usize;

        Some(r + last_block_rank)
    }
}

impl Select for RRRBitVec {
    #[inline]
    fn select(&self, rank: usize) -> Option<usize> {
        if rank == 0 {
            return Some(0);
        }
        if rank > self.total_rank {
            return None;
        }

        let mut left = 0;
        let mut right = self.rank_samples.len() - 1;

        while right - left > 1 {
            let mid = (left + right) / 2;
            let mid_rank = unsafe { self.rank_samples.get_unchecked(mid) };
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
        let mut local_rank = unsafe { self.rank_samples.get_unchecked(left) };
        let mut local_pos = unsafe { self.offset_samples.get_unchecked(left) };
        let mut block_index = left * self.k;
        let mut class = unsafe { self.classes.get_unchecked(block_index) };

        // Stop where at the next block step, the rank is greater than the rank we are looking for
        while local_rank + class < rank {
            local_rank += class;
            local_pos += unsafe { self.lengths.get_unchecked(class) };
            block_index += 1;

            class = unsafe { self.classes.get_unchecked(block_index) };
        }

        // at this point, we are exactly that the block that contains the rank is `block_index`
        let class_length = unsafe { *self.lengths.get_unchecked(class) };
        let offset = unsafe { self.offsets.read_bits_unchecked(local_pos, class_length) };

        let mut block = Self::decode(class, offset, self.b, self.b);

        // select the bit in the block
        let mut bit_index = 0;
        while local_rank < rank {
            if block & 0b1 == 0b1 {
                local_rank += 1;
            }
            block >>= 1;
            bit_index += 1;
        }

        Some(block_index * self.b + bit_index)
    }

    #[inline]
    fn select0(&self, rank0: usize) -> Option<usize> {
        if rank0 == 0 {
            return Some(0);
        }
        let total_rank0 = self.len() - self.total_rank;
        if rank0 > total_rank0 {
            return None;
        }

        let mut left = 0;
        let mut right = self.rank_samples.len() - 1;

        while right - left > 1 {
            let mid = (left + right) / 2;
            let bits_before_mid = mid * self.b * self.k;
            let mid_rank0 = bits_before_mid - unsafe { self.rank_samples.get_unchecked(mid) };
            if mid_rank0 < rank0 {
                left = mid;
            } else {
                right = mid;
            }
        }

        // search for the block that contains the rank
        let bits_before_left = left * self.b * self.k;
        let mut local_rank0 = bits_before_left - unsafe { self.rank_samples.get_unchecked(left) };
        let mut local_pos = unsafe { self.offset_samples.get_unchecked(left) };
        let mut block_index = left * self.k;
        let mut class = unsafe { self.classes.get_unchecked(block_index) };

        // Stop where at the next block step, the rank is greater than the rank we are looking for
        // (self.b - class) is the number of 0s in the block
        while local_rank0 + (self.b - class) < rank0 {
            local_rank0 += self.b - class;
            local_pos += unsafe { self.lengths.get_unchecked(class) };
            block_index += 1;
            class = unsafe { self.classes.get_unchecked(block_index) };
        }

        // at this point, we are exactly that the block that contains the rank is `block_index`
        let class_length = unsafe { *self.lengths.get_unchecked(class) };
        let offset = unsafe { self.offsets.read_bits_unchecked(local_pos, class_length) };

        let mut block = Self::decode(class, offset, self.b, self.b);

        // select the bit in the block
        let mut bit_index = 0;
        while local_rank0 < rank0 {
            if block & 0b1 == 0b0 {
                local_rank0 += 1;
            }
            block >>= 1;
            bit_index += 1;
        }

        Some(block_index * self.b + bit_index)
    }
}

impl Access for RRRBitVec {
    #[inline]
    fn access(&self, index: usize) -> Option<bool> {
        if index >= self.len() {
            return None;
        }
        Some(self.read(index))
    }
}

pub struct RRRBitVecSpec {
    pub b: usize,
    pub k: usize,
}

impl RRRBitVecSpec {
    #[inline]
    pub const fn new(b: usize, k: usize) -> Self {
        Self { b, k }
    }
}

impl RRRBitVec {
    #[inline]
    pub const fn spec(b: usize, k: usize) -> RRRBitVecSpec {
        RRRBitVecSpec::new(b, k)
    }
}

impl Build<BitVec, RRRBitVec> for RRRBitVecSpec {
    #[inline]
    fn build(&self, data: BitVec) -> RRRBitVec {
        RRRBitVec::new(data, self.b, self.k)
    }
}

#[cfg(test)]
mod tests;
