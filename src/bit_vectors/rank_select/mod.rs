//! Rank/Select data structures.

/// Trait that defines structures that themselves full support rank.
pub trait Rank {
    /// Returns the number of 1s in the bit vector in the range [0, index).
    /// If index is out of bounds, returns None.
    /// By definition, rank(0) == 0
    fn rank(&self, index: usize) -> Option<usize>;

    /// Returns the number of 0s in the bit vector in the range [0, index).
    /// If index is out of bounds, returns None.
    /// By definition, rank0(0) == 0
    fn rank0(&self, index: usize) -> Option<usize> {
        self.rank(index).map(|rank| index - rank)
    }
}

pub trait Select {
    /// Return the index of the i-th 1 in the bit vector.
    /// select(0) == 0
    fn select(&self, rank: usize) -> Option<usize>;

    /// Return the index of the i-th 0 in the bit vector.
    fn select0(&self, rank0: usize) -> Option<usize>;
}

/// Trait that defines structures that support rank along with the bit vector.
pub trait RankSupport<T> {
    /// Returns the number of 1s in the bit vector in the range [0, index).
    /// # Safety
    /// The data used must be the same data that the structure was built with.
    unsafe fn rank(&self, data: &T, index: usize) -> Option<usize>;

    /// Returns the number of 0s in the bit vector in the range [0, index).
    /// # Safety
    /// The data used must be the same data that the structure was built with.
    unsafe fn rank0(&self, data: &T, index: usize) -> Option<usize> {
        self.rank(data, index).map(|rank| index - rank)
    }
}

pub trait SelectSupport<T> {
    /// Return the index of the i-th 1 in the bit vector.
    /// select(0) == 0
    /// # Safety
    /// The data used must be the same data that the structure was built with.
    unsafe fn select(&self, data: &T, rank: usize) -> Option<usize>;

    /// Return the index of the i-th 0 in the bit vector.
    /// # Safety
    /// The data used must be the same data that the structure was built with.
    unsafe fn select0(&self, data: &T, rank0: usize) -> Option<usize>;
}

#[derive(Debug)]
pub struct RankStructure<T, R>
where
    R: RankSupport<T>,
{
    data: T,
    rank_support: R,
}

impl<T, R> RankStructure<T, R>
where
    R: RankSupport<T>,
{
    /// # Safety
    /// The data used must be the same data that the structure was built with.
    pub unsafe fn new(data: T, rank_support: R) -> Self {
        Self { data, rank_support }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn rank_support(&self) -> &R {
        &self.rank_support
    }
}

impl<T, R> Rank for RankStructure<T, R>
where
    R: RankSupport<T>,
{
    fn rank(&self, index: usize) -> Option<usize> {
        unsafe { self.rank_support.rank(&self.data, index) }
    }
}

/// Weak select implementation that uses the rank structure
impl<T, R> Select for RankStructure<T, R>
where
    R: SelectSupport<T> + RankSupport<T>,
{
    fn select(&self, rank: usize) -> Option<usize> {
        unsafe { self.rank_support.select(&self.data, rank) }
    }

    fn select0(&self, rank0: usize) -> Option<usize> {
        unsafe { self.rank_support.select0(&self.data, rank0) }
    }
}

impl<T, R> Access for RankStructure<T, R>
where
    T: Access,
    R: RankSupport<T>,
{
    fn access(&self, index: usize) -> Option<bool> {
        self.data.access(index)
    }
}

impl<T, R> HeapSize for RankStructure<T, R>
where
    T: HeapSize,
    R: HeapSize + RankSupport<T>,
{
    fn heap_size_in_bits(&self) -> usize {
        self.data.heap_size_in_bits() + self.rank_support.heap_size_in_bits()
    }
}

#[derive(Debug)]
pub struct SelectStructure<T, S>
where
    S: SelectSupport<T>,
{
    data: T,
    select_support: S,
}

impl<T, S> SelectStructure<T, S>
where
    S: SelectSupport<T>,
{
    /// # Safety
    /// The data used must be the same data that the structure was built with.
    pub unsafe fn new(data: T, select_support: S) -> Self {
        Self {
            data,
            select_support,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn select_support(&self) -> &S {
        &self.select_support
    }
}

impl<T, S> Select for SelectStructure<T, S>
where
    S: SelectSupport<T>,
{
    fn select(&self, rank: usize) -> Option<usize> {
        unsafe { self.select_support.select(&self.data, rank) }
    }
    fn select0(&self, rank: usize) -> Option<usize> {
        unsafe { self.select_support.select0(&self.data, rank) }
    }
}

/// Weak select implementation that uses the rank structure
impl<T, S> Rank for SelectStructure<T, S>
where
    S: SelectSupport<T> + RankSupport<T>,
{
    fn rank(&self, index: usize) -> Option<usize> {
        unsafe { self.select_support.rank(&self.data, index) }
    }
}

impl<T, S> Access for SelectStructure<T, S>
where
    T: Access,
    S: SelectSupport<T>,
{
    fn access(&self, index: usize) -> Option<bool> {
        self.data.access(index)
    }
}

impl<T, S> HeapSize for SelectStructure<T, S>
where
    T: HeapSize,
    S: HeapSize + SelectSupport<T>,
{
    fn heap_size_in_bits(&self) -> usize {
        self.data.heap_size_in_bits() + self.select_support.heap_size_in_bits()
    }
}

#[derive(Debug)]
pub struct RankSelectStructure<T, R, S>
where
    R: RankSupport<T>,
    S: SelectSupport<T>,
{
    data: T,
    rank_support: R,
    select_support: S,
}

impl<T, R, S> RankSelectStructure<T, R, S>
where
    R: RankSupport<T>,
    S: SelectSupport<T>,
{
    /// # Safety
    /// The data used must be the same data that the structure was built with.
    pub unsafe fn new(data: T, rank_support: R, select_support: S) -> Self {
        Self {
            data,
            rank_support,
            select_support,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn rank_support(&self) -> &R {
        &self.rank_support
    }

    pub fn select_support(&self) -> &S {
        &self.select_support
    }
}

impl<T, R, S> Rank for RankSelectStructure<T, R, S>
where
    R: RankSupport<T>,
    S: SelectSupport<T>,
{
    fn rank(&self, index: usize) -> Option<usize> {
        unsafe { self.rank_support.rank(&self.data, index) }
    }
}

impl<T, R, S> Select for RankSelectStructure<T, R, S>
where
    R: RankSupport<T>,
    S: SelectSupport<T>,
{
    fn select(&self, rank: usize) -> Option<usize> {
        unsafe { self.select_support.select(&self.data, rank) }
    }

    fn select0(&self, rank0: usize) -> Option<usize> {
        unsafe { self.select_support.select0(&self.data, rank0) }
    }
}

impl<T, R, S> Access for RankSelectStructure<T, R, S>
where
    T: Access,
    S: SelectSupport<T>,
    R: RankSupport<T>,
{
    fn access(&self, index: usize) -> Option<bool> {
        self.data.access(index)
    }
}

impl<T, R, S> HeapSize for RankSelectStructure<T, R, S>
where
    T: HeapSize,
    S: HeapSize + SelectSupport<T>,
    R: HeapSize + RankSupport<T>,
{
    fn heap_size_in_bits(&self) -> usize {
        self.data.heap_size_in_bits()
            + self.rank_support.heap_size_in_bits()
            + self.select_support.heap_size_in_bits()
    }
}

// Build RankSelectStructure in a generic way, using a RankStructure and a SelectStructure
// This allow us to use different structures to support these operations, in a centralized
// and error-free way.

impl<T, R, S, B> Build<RankStructure<T, R>, RankSelectStructure<T, R, S>> for B
where
    R: RankSupport<T>,
    B: Build<T, SelectStructure<T, S>>,
    S: SelectSupport<T>,
{
    fn build(&self, data: RankStructure<T, R>) -> RankSelectStructure<T, R, S> {
        let rank_support = data.rank_support;
        let data = data.data;
        let select_structure = self.build(data);
        let select_support = select_structure.select_support;
        let data = select_structure.data;

        unsafe { RankSelectStructure::new(data, rank_support, select_support) }
    }
}

impl<T, R, S, B> Build<SelectStructure<T, S>, RankSelectStructure<T, R, S>> for B
where
    R: RankSupport<T>,
    B: Build<T, RankStructure<T, R>>,
    S: SelectSupport<T>,
{
    fn build(&self, data: SelectStructure<T, S>) -> RankSelectStructure<T, R, S> {
        let select_support = data.select_support;
        let data = data.data;
        let rank_structure = self.build(data);
        let rank_support = rank_structure.rank_support;
        let data = rank_structure.data;

        unsafe { RankSelectStructure::new(data, rank_support, select_support) }
    }
}

pub trait Build<T, O> {
    fn build(&self, data: T) -> O;
}

// pub trait RankSelect: Rank + Select {}

// impl<T> RankSelect for T where T: Rank + Select {}

pub mod sparse_sampling_rank;
pub use sparse_sampling_rank::SparseSamplingRank;

pub mod dense_sampling_rank;
pub use dense_sampling_rank::DenseSamplingRank;

// TODO: not implemented yet
// pub mod sparse_sampling_select;

use crate::profiling::HeapSize;

use super::Access;

#[cfg(test)]
pub(crate) mod tests_utils;
