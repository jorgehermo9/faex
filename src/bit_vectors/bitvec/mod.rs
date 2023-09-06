use super::rank_select::{Rank, Select};
use super::Access;
use crate::profiling::HeapSize;
use crate::util::{ceil_div, getbits, setbits};
use crate::Build;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;

/// Data Structure that represents a bit vector using a compact storage.
// TODO: Be generic over the underlying data container? This could allow to retrieve
// ints of size greater than u64 (such as u128 or other integer types that could be in the future)

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BitVec {
    /// The underlying data structure
    raw_data: Vec<usize>,
    len: usize,
}

impl Debug for BitVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let raw_data_formatted = self
            .raw_data
            .iter()
            .map(|word| format!("{word:b}"))
            .collect::<Vec<String>>();

        f.debug_struct("BitVec")
            .field("raw_data", &raw_data_formatted)
            .field("len", &self.len)
            .finish()
    }
}

impl BitVec {
    pub const CONTAINER_WIDTH: usize = std::mem::size_of::<usize>() * 8;

    #[inline]
    pub fn new() -> Self {
        Self {
            raw_data: Vec::new(),
            len: 0,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        // ceil division with usize
        let compact_capacity = ceil_div(capacity, BitVec::CONTAINER_WIDTH);
        Self {
            raw_data: Vec::with_capacity(compact_capacity),
            len: 0,
        }
    }

    #[inline]
    pub fn from_value(value: bool, n: usize) -> Self {
        let num_words = ceil_div(n, BitVec::CONTAINER_WIDTH);
        let mut raw_data = Vec::with_capacity(num_words);
        let value = if value { usize::MAX } else { 0 };
        raw_data.resize(num_words, value);

        // clear unused bits
        let last_word_offset = n % BitVec::CONTAINER_WIDTH;
        if last_word_offset != 0 {
            let last_word = raw_data.last_mut().unwrap();
            *last_word &= usize::MAX >> (BitVec::CONTAINER_WIDTH - last_word_offset);
        }

        Self { raw_data, len: n }
    }

    // TODO: create a method to create a BitVec with a bool value
    // repeated n times, or just use from_iter method? (with collect)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw_data.capacity() * BitVec::CONTAINER_WIDTH
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw_data.is_empty()
    }

    #[inline]
    pub fn raw_data(&self) -> &[usize] {
        &self.raw_data
    }

    #[inline]
    pub fn push(&mut self, value: bool) {
        let (block_index, offset) = (
            self.len / BitVec::CONTAINER_WIDTH,
            self.len % BitVec::CONTAINER_WIDTH,
        );

        let value = value.into();

        if offset == 0 {
            self.raw_data.push(value);
        } else {
            setbits!(self.raw_data[block_index], 1, offset, value);
        }
        self.len += 1;
    }

    /// Pushes a value into the bit vector, using the specified width.
    /// The bits will be read in a LSB-first order (i.e from right to left).
    // TODO: hardcode T as usize? then some asserts can be removed
    // TODO: panic or return result explaining the error? Maybe results would decrease performance
    #[inline]
    pub fn push_bits<T>(&mut self, value: T, width: usize)
    where
        T: Into<usize>,
    {
        // Allow to push a value of size less than the width, the bits will
        // be padded with 0s
        let value = value.into();

        // This check prevents from a single value span more than two words
        assert!(
            width <= BitVec::CONTAINER_WIDTH,
            "Width {width} is greater than the BitVec's container width ({})",
            BitVec::CONTAINER_WIDTH
        );

        // TODO: return or crash?
        if width == 0 {
            return;
        }
        let (block_index, offset) = (
            self.len / BitVec::CONTAINER_WIDTH,
            self.len % BitVec::CONTAINER_WIDTH,
        );

        if offset == 0 {
            self.raw_data.push(value);
        } else {
            let last_block = self.raw_data[block_index];
            let shifted_value = value << offset;
            let new_value = shifted_value | last_block;
            self.raw_data[block_index] = new_value;

            let filled_bits = BitVec::CONTAINER_WIDTH - offset;

            if filled_bits < width {
                let remaining_value = value >> filled_bits;
                self.raw_data.push(remaining_value);
            }
        }
        self.len += width;
    }

    #[inline]
    pub fn pop(&mut self) -> bool {
        assert!(self.len > 0, "Cannot pop a bit from an empty BitVec");

        let value = self.read(self.len - 1);

        // clear bit at pos len -1 so there are no dirty bits
        self.set(self.len - 1, false);
        self.len -= 1;

        if self.len % BitVec::CONTAINER_WIDTH == 0 {
            // If that bit was the last one in the last word, remove that word
            self.raw_data.pop();
        }
        value
    }

    #[inline]
    pub fn pop_bits(&mut self, n: usize) -> usize {
        assert!(
            n <= self.len,
            "Cannot pop {n} bits from a BitVec of length {}",
            self.len,
        );

        let new_len = self.len - n;
        let value = self.read_bits(new_len, n);

        let old_compact_len = ceil_div(self.len, BitVec::CONTAINER_WIDTH);
        let new_compact_len = ceil_div(new_len, BitVec::CONTAINER_WIDTH);

        if old_compact_len > new_compact_len {
            self.raw_data.truncate(new_compact_len);
        }

        // clear the bits of the last word that are not used
        let last_word_offset = new_len % BitVec::CONTAINER_WIDTH;
        if last_word_offset != 0 {
            let last_word = self.raw_data.last_mut().unwrap();
            *last_word &= usize::MAX >> (BitVec::CONTAINER_WIDTH - last_word_offset);
        }

        self.len = new_len;

        value
    }

    #[inline]
    pub fn read(&self, index: usize) -> bool {
        assert!(
            index < self.len,
            "Cannot read bit at index {index} from a BitVec of length {}",
            self.len
        );

        let (block_index, offset) = (
            index / BitVec::CONTAINER_WIDTH,
            index % BitVec::CONTAINER_WIDTH,
        );

        // TODO: use get_unchecked to avoid bounds check, since
        // we are already checking bounds.

        // Optimize this as we know we are reading 1 bit
        // getbits!(self.raw_data[block_index], 1, offset) == 1
        self.raw_data[block_index] >> offset & 0b1 == 1
    }

    #[inline]
    pub fn read_bits(&self, index: usize, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        assert!(
            len <= BitVec::CONTAINER_WIDTH,
            "requested len ({len}) is greater than the BitVec's container width ({})",
            BitVec::CONTAINER_WIDTH
        );

        assert!(
            index + len - 1 < self.len(),
            "index out of bounds: the len is {}, but the index is {} and the width is {}",
            self.len(),
            index,
            len
        );
        unsafe { self.read_bits_unchecked(index, len) }
    }

    /// # Safety
    /// Calling this method with an out-of-bounds `index + len` is undefined behavior.
    #[inline]
    pub unsafe fn read_bits_unchecked(&self, index: usize, len: usize) -> usize {
        // TODO: optimize this method to achieve better performance
        if len == 0 {
            return 0;
        }
        let offset = index % BitVec::CONTAINER_WIDTH;
        let index = index / BitVec::CONTAINER_WIDTH;

        let w1 = self.raw_data.get_unchecked(index) >> offset;
        if offset + len > BitVec::CONTAINER_WIDTH {
            let w2 = self.raw_data.get_unchecked(index + 1);
            let read_bits = BitVec::CONTAINER_WIDTH - offset;
            let rem_bits = len - read_bits;
            w1 | ((w2 & (usize::MAX >> (BitVec::CONTAINER_WIDTH - rem_bits))) << read_bits)
        } else {
            w1 & (usize::MAX >> (BitVec::CONTAINER_WIDTH - len))
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        assert!(
            index < self.len,
            "Cannot set bit at index {index} from a BitVec of length {}",
            self.len
        );

        let (block_index, offset) = (
            index / BitVec::CONTAINER_WIDTH,
            index % BitVec::CONTAINER_WIDTH,
        );
        let value: usize = value.into();

        setbits!(self.raw_data[block_index], 1, offset, value);
    }

    #[inline]
    pub fn set_bits<T>(&mut self, range: Range<usize>, value: T)
    where
        T: Into<usize>,
    {
        if range.start == range.end {
            return;
        }

        let local_container_width: usize = std::mem::size_of::<T>() * 8;
        let value = value.into();

        let start = range.start;
        let end = range.end - 1;
        let width = range.end - range.start;

        assert!(
            end < self.len,
            "Cannot set bits from index {start} to {end} from a BitVec of length {}",
            self.len
        );

        assert!(
            width <= BitVec::CONTAINER_WIDTH,
            "Range's width {width} is greater than the BitVec's container width ({})",
            BitVec::CONTAINER_WIDTH
        );

        assert!(
            width <= local_container_width,
            "Range's width {width} is greater than the provided value's bits ({local_container_width})"
        );

        if width == 0 {
            return;
        }

        let first_index = start / BitVec::CONTAINER_WIDTH;
        let first_offset = start % BitVec::CONTAINER_WIDTH;

        let last_index = end / BitVec::CONTAINER_WIDTH;
        if first_index == last_index {
            // when the int is contained in a single word
            setbits!(self.raw_data[first_index], width, first_offset, value);
        } else {
            //when the int is split between two words
            let first_word_num_bits = BitVec::CONTAINER_WIDTH - first_offset;
            let first_value = getbits!(value, first_word_num_bits, 0);
            setbits!(
                self.raw_data[first_index],
                first_word_num_bits,
                first_offset,
                first_value
            );

            let last_word_num_bits = width - first_word_num_bits;
            let last_value = getbits!(value, last_word_num_bits, first_word_num_bits);
            setbits!(self.raw_data[last_index], last_word_num_bits, 0, last_value);
        }
    }

    // TODO: create a remove function similar to Vec's api
}

impl Default for BitVec {
    fn default() -> Self {
        Self::new()
    }
}

impl HeapSize for BitVec {
    fn heap_size_in_bits(&self) -> usize {
        self.raw_data.heap_size_in_bits()
    }
}

impl Access for BitVec {
    fn access(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        Some(self.read(index))
    }
}

impl Rank for BitVec {
    fn rank(&self, index: usize) -> Option<usize> {
        // rank is the number of 1s before the index (range [0, index))
        if index > self.len {
            return None;
        }

        let (block_index, block_offset) = (
            index / BitVec::CONTAINER_WIDTH,
            index % BitVec::CONTAINER_WIDTH,
        );

        let prev_blocks = &self.raw_data[..block_index];
        let prev_rank = prev_blocks
            .iter()
            .map(|block| block.count_ones())
            .sum::<u32>() as usize;

        let last_block = self.raw_data.get(block_index).copied().unwrap_or(0);
        // getbits!(last_block, block_offset, 0) optimized, we know that block_offset is always <64 so this does not overflow
        let last_block_target = last_block & ((1 << block_offset) - 1);
        let last_block_rank = last_block_target.count_ones() as usize;

        Some(prev_rank + last_block_rank)
    }
}

impl Select for BitVec {
    fn select(&self, rank: usize) -> Option<usize> {
        if rank == 0 {
            return Some(0);
        }

        let mut rank_count = 0;
        for (i, block) in self.raw_data.iter().enumerate() {
            let block_rank = block.count_ones() as usize;
            if rank_count + block_rank >= rank {
                // Select inside word
                let mut bit_index = 0;
                let mut word = *block;
                while rank_count < rank {
                    let bit = word & 0b1;
                    if bit == 1 {
                        rank_count += 1;
                    }
                    word >>= 1;
                    bit_index += 1;
                }
                return Some(i * BitVec::CONTAINER_WIDTH + bit_index);
            }
            rank_count += block_rank;
        }
        None
    }

    fn select0(&self, rank0: usize) -> Option<usize> {
        if rank0 == 0 {
            return Some(0);
        }

        let mut rank0_count = 0;
        for (i, block) in self.raw_data.iter().enumerate() {
            let block_rank0 = if i == self.raw_data.len() - 1 {
                // At the last word, it may not be fully completed and there may be dirty bits
                // at value 0, so we do not count them
                (self.len - i * BitVec::CONTAINER_WIDTH) - block.count_ones() as usize
            } else {
                block.count_zeros() as usize
            };

            if rank0_count + block_rank0 >= rank0 {
                // Select inside word
                let mut bit_index = 0;
                let mut word = *block;
                while rank0_count < rank0 {
                    let bit = word & 0b1;
                    if bit == 0 {
                        rank0_count += 1;
                    }
                    word >>= 1;
                    bit_index += 1;
                }
                return Some(i * BitVec::CONTAINER_WIDTH + bit_index);
            }
            rank0_count += block_rank0;
        }

        None
    }
}

impl Display for BitVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.iter();
        write!(f, "[")?;
        if let Some(bit) = iter.next() {
            let bit = if bit { 1 } else { 0 };
            write!(f, "{bit}")?;
        }
        for bit in iter {
            let bit = if bit { 1 } else { 0 };
            write!(f, ", {bit}")?;
        }
        write!(f, "]")
    }
}

pub struct BitVecSpec;

impl BitVecSpec {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for BitVecSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl BitVec {
    pub const fn spec() -> BitVecSpec {
        BitVecSpec::new()
    }
}

impl Build<BitVec, BitVec> for BitVecSpec {
    fn build(&self, data: BitVec) -> BitVec {
        data
    }
}

pub mod iter;
#[cfg(test)]
mod tests;
