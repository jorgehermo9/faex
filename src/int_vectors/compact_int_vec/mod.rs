use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{bit_vectors::BitVec, profiling::HeapSize};

/// Data structure that stores a sequence of integers of a fixed width in a compact way.
/// The underlying data structure is a bit vector, which bits are interpreted together to store integers.
/// The width of the integers is specified when creating the data structure and cannot be modified.
/// The width must be at most the width of usize, since it is required for the BitVec data structure.

// TODO: implement pretty to string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactIntVec {
    raw_data: BitVec,
    width: usize,
    len: usize,
}

impl CompactIntVec {
    // TODO: create a from raw parts method that allos us to interpretate a bit vector
    // as a vector of ints. This method should be called from_raw_parts,
    // or from_bitvec? we should only specify the width of the integers,
    // since len is calculated with ceil_div(bitvec.len(), width)

    #[inline]
    pub fn new(width: usize) -> Self {
        Self::validate_width(width);
        Self {
            raw_data: BitVec::new(),
            width,
            len: 0,
        }
    }

    #[inline]
    pub fn from_raw_parts(mut raw_data: BitVec, width: usize) -> Self {
        Self::validate_width(width);

        let leftover_bits = raw_data.len() % width;
        if leftover_bits != 0 {
            // fill the last integer with zeros in order to have a complete integer
            raw_data.push_bits(0usize, width - (leftover_bits));
        }
        let len = raw_data.len() / width;

        Self {
            raw_data,
            width,
            len,
        }
    }

    #[inline]
    pub fn with_capacity(width: usize, capacity: usize) -> Self {
        Self::validate_width(width);

        Self {
            raw_data: BitVec::with_capacity(width * capacity),
            width,
            len: 0,
        }
    }

    #[inline]
    fn validate_width(width: usize) {
        assert!(
            width <= BitVec::CONTAINER_WIDTH,
            "The width of the integers must be at most {} bits, got {width} bits",
            BitVec::CONTAINER_WIDTH
        );
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw_data
            .capacity()
            .checked_div(self.width)
            // if width is 0, we can allocate usize::MAX elements
            .unwrap_or(usize::MAX)
    }

    #[inline]
    pub fn raw_data(&self) -> &BitVec {
        &self.raw_data
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn push<T>(&mut self, value: T)
    where
        T: Into<usize>,
    {
        let value: usize = value.into();
        assert!(
            value.checked_shr(self.width as u32).unwrap_or(0) == 0,
            "Value {value} does not fit within {} bits",
            self.width
        );

        self.raw_data.push_bits(value, self.width);
        self.len += 1;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        self.len -= 1;
        Some(self.raw_data.pop_bits(self.width))
    }

    /// When width==0, get of any index will return 0
    #[inline]
    pub fn get(&self, index: usize) -> Option<usize> {
        if index >= self.len() {
            return None;
        }
        Some(unsafe { self.get_unchecked(index) })
    }

    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> usize {
        self.raw_data
            .read_bits_unchecked(index * self.width, self.width)
    }

    #[inline]
    pub fn set<T>(&mut self, index: usize, value: T)
    where
        T: Into<usize>,
    {
        let value: usize = value.into();
        assert!(
            value.checked_shr(self.width as u32).unwrap_or(0) == 0,
            "Value {value} does not fit within {} bits",
            self.width
        );

        let range = index * self.width..(index + 1) * self.width;
        self.raw_data.set_bits(range, value);
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

impl HeapSize for CompactIntVec {
    fn heap_size_in_bits(&self) -> usize {
        self.raw_data.heap_size_in_bits()
    }
}

impl Display for CompactIntVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.iter();
        write!(f, "[")?;
        if let Some(value) = iter.next() {
            write!(f, "{}", value)?;
        }
        for value in iter {
            write!(f, ", {}", value)?;
        }
        write!(f, "]")
    }
}

pub mod iter;

#[cfg(test)]
mod tests;
