use std::fmt::{Debug, Formatter};

use crate::{bit_vectors::BitVec, profiling::HeapSize};


// TODO: implement clone for this struct
pub struct VariableSizeIntVec<'a> {
    raw_data: BitVec,
    len: usize,
    size_function: Box<dyn Fn(usize) -> usize + 'a>,
    k: usize,
    samples: Vec<usize>,
}

impl Debug for VariableSizeIntVec<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VariableSizeIntVec")
            .field("raw_data", &self.raw_data)
            .field("len", &self.len)
            .field("k", &self.k)
            .field("samples", &self.samples)
            .finish()
    }
}

impl<'a> VariableSizeIntVec<'a> {
    /// max pos sample value must be known in order to compact samples
    #[inline]
    pub fn new<F>(size_function: F, k: usize) -> Self
    where
        F: Fn(usize) -> usize + 'a,
    {
        Self {
            raw_data: BitVec::new(),
            len: 0,
            size_function: Box::new(size_function),
            k,
            samples: Vec::new(),
        }
    }

    #[inline]
    fn validate_size(size: usize) {
        assert!(
            size <= BitVec::CONTAINER_WIDTH,
            "The size of the integers must be at most {} bits, got {size} bits",
            BitVec::CONTAINER_WIDTH
        );
    }

    #[inline]
    pub fn push<T>(&mut self, value: T)
    where
        T: Into<usize>,
    {
        let value: usize = value.into();
        let size = (self.size_function)(self.len());
        Self::validate_size(size);

        assert!(
            value.checked_shr(size as u32).unwrap_or(0) == 0,
            "Value {value} does not fit within {size} bits",
        );

        // Store the prev_sizes up to the current element
        if self.len() % self.k == 0 {
            self.samples.push(self.raw_data.len());
        }

        self.raw_data.push_bits(value, size);

        self.len += 1;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        let index = self.len() - 1;

        let size = (self.size_function)(index);

        if index % self.k == 0 {
            self.samples.pop();
        }

        self.len -= 1;
        Some(self.raw_data.pop_bits(size))
    }

    /// If the index is out of bounds, return 0.
    #[inline]
    pub fn get(&self, index: usize) -> usize {
        assert!(
            index < self.len(),
            "index out of bounds: the len is {} but the index is {index}",
            self.len(),
        );

        unsafe { self.get_unchecked(index) }
    }

    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> usize {
        let size = (self.size_function)(index);
        let sample_index = index / self.k;
        let sampled_position = *self.samples.get_unchecked(sample_index);

        let mut pos = sampled_position;
        for i in sample_index * self.k..index {
            pos += (self.size_function)(i)
        }

        // TODO: as we are checking bounds, we may use a
        // bitvec function that does not check that bounds.
        // but we must use unsafe
        // self.raw_data
        //     .read_bits(start_position..start_position + size)
        self.raw_data.read_bits_unchecked(pos, size)
    }

    #[inline]
    pub fn checked_get(&self, index: usize) -> Option<usize> {
        if index >= self.len() {
            return None;
        }

        Some(self.get(index))
    }

    #[inline]
    pub fn set<T>(&mut self, index: usize, value: T)
    where
        T: Into<usize>,
    {
        let size = (self.size_function)(index);
        let value = value.into();

        assert!(
            index < self.len(),
            "index out of bounds: the len is {} but the index is {index}",
            self.len(),
        );

        assert!(
            value.checked_shr(size as u32).unwrap_or(0) == 0,
            "Value {value} does not fit within {size} bits",
        );

        let sample_index = index / self.k;
        let sampled_position = self.samples.get(sample_index).unwrap();

        // At most, we will have to count length of k-1 elements to find
        // where the element at `index` starts
        let start_position = sampled_position
            + (sample_index * self.k..index)
                .map(&self.size_function)
                .sum::<usize>();

        self.raw_data
            .set_bits(start_position..start_position + size, value);
    }

    #[inline]
    pub fn raw_data(&self) -> &BitVec {
        &self.raw_data
    }

    #[inline]
    pub fn size_function(&self) -> &dyn Fn(usize) -> usize {
        &*self.size_function
    }

    #[inline]
    pub fn k(&self) -> usize {
        self.k
    }

    #[inline]
    pub fn samples(&self) -> &[usize] {
        &self.samples
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // TODO: optimize function that constructs a new vector but
    // with a CompactIntVec for samples.
    // Do not use method CompactIntVec::from, since we already know that
    // the max value of the samples will be on the last one.
}

impl HeapSize for VariableSizeIntVec<'_> {
    fn heap_size_in_bits(&self) -> usize {
        self.raw_data.heap_size_in_bits() + self.samples.heap_size_in_bits()
    }
}

pub mod iter;

#[cfg(test)]
mod tests;
