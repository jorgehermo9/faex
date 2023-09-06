use std::{borrow::Cow, collections::HashSet, fmt::Debug, fs::File, io::Read, marker::PhantomData};

use crate::Build;
use crate::{
    bit_vectors::{
        rank_select::{Rank, Select},
        Access, BitVec,
    },
    profiling::HeapSize,
};

use super::{CharacterAccess, CharacterRank, CharacterSelect};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WaveletTree<T> {
    alphabet: Vec<char>,
    root: WaveletTreeNode<T>,
    len: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WaveletTreeNode<T> {
    Internal {
        left: Box<WaveletTreeNode<T>>,
        right: Box<WaveletTreeNode<T>>,
        bit_vec: T,
    },
    Leaf {
        len: usize,
    },
}

impl<T> WaveletTree<T> {
    /// Builds a wavelet tree from the given string data. The alphabet is inferred from the data.
    ///
    /// # Panics
    ///
    /// This function panics if the string data is empty.
    #[inline]
    pub fn new<B>(data: &str, spec: &B) -> Self
    where
        B: Build<BitVec, T>,
    {
        assert!(!data.is_empty(), "string data cannot be empty");

        let mut alphabet = data
            .chars()
            .collect::<HashSet<_>>()
            .iter()
            .copied()
            .collect::<Vec<_>>();

        // Do not use BTreeSet because it is slow, better to do it this way
        alphabet.sort_unstable();

        let len = data.chars().count();

        let root = Self::build_tree(Cow::from(data), &alphabet, spec);

        Self {
            alphabet,
            root,
            len,
        }
    }

    #[inline]
    fn build_tree<B>(data: Cow<'_, str>, alphabet: &[char], spec: &B) -> WaveletTreeNode<T>
    where
        B: Build<BitVec, T>,
    {
        if alphabet.len() == 1 {
            // It is not necessary to store the bitvec in the leaf, since it will be all 1s
            return WaveletTreeNode::Leaf {
                len: data.chars().count(),
            };
        }

        let mid = (alphabet.len() - 1) / 2;
        let left_alphabet = &alphabet[..=mid];
        let right_alphabet = &alphabet[mid + 1..];
        let mid_char = alphabet[mid];

        let bit_vec = data.chars().map(|c| c > mid_char).collect::<BitVec>();

        let (left_data, right_data): (String, String) = data.chars().partition(|c| *c <= mid_char);
        let left = Self::build_tree(Cow::from(left_data), left_alphabet, spec);
        let right = Self::build_tree(Cow::from(right_data), right_alphabet, spec);

        WaveletTreeNode::Internal {
            left: Box::new(left),
            right: Box::new(right),
            bit_vec: spec.build(bit_vec),
        }
    }

    #[inline]
    pub fn contains(&self, char: &char) -> bool {
        self.alphabet.binary_search(char).is_ok()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn alphabet(&self) -> &[char] {
        &self.alphabet
    }

    #[inline]
    pub fn root(&self) -> &WaveletTreeNode<T> {
        &self.root
    }
}

impl<T> CharacterRank for WaveletTree<T>
where
    T: Rank,
{
    #[inline]
    fn rank(&self, char: char, index: usize) -> Option<usize> {
        if index > self.len {
            return None;
        }

        let mut node = &self.root;
        let mut index = index;
        let mut interval_left = 0;
        let mut interval_right = self.alphabet.len() - 1;

        while let WaveletTreeNode::Internal {
            left,
            right,
            bit_vec,
        } = node
        {
            let mid = (interval_left + interval_right) / 2;
            let mid_char = unsafe { self.alphabet.get_unchecked(mid) };
            if char <= *mid_char {
                node = left;
                interval_right = mid;
                index = bit_vec.rank0(index).unwrap();
            } else {
                node = right;
                interval_left = mid + 1;
                index = bit_vec.rank(index).unwrap();
            }
        }

        let leaf_char = unsafe { self.alphabet.get_unchecked(interval_left) };
        if *leaf_char != char {
            return None;
        }

        // When we are at the leaf node, the rank is the index
        Some(index)
    }
}

impl<T> CharacterAccess for WaveletTree<T>
where
    T: Rank + Access,
{
    fn access(&self, index: usize) -> Option<char> {
        if index >= self.len {
            return None;
        }

        let mut node = &self.root;
        let mut index = index;
        let mut interval_left = 0;
        let mut interval_right = self.alphabet.len() - 1;

        while let WaveletTreeNode::Internal {
            left,
            right,
            bit_vec,
        } = node
        {
            let mid = (interval_left + interval_right) / 2;
            if bit_vec.access(index).unwrap() {
                node = right;
                interval_left = mid + 1;
                index = bit_vec.rank(index).unwrap();
            } else {
                node = left;
                interval_right = mid;
                index = bit_vec.rank0(index).unwrap();
            }
        }

        // At the leaf node, interval_left == interval_right
        let leaf_char = unsafe { *self.alphabet.get_unchecked(interval_left) };
        Some(leaf_char)
    }
}

impl<T> CharacterSelect for WaveletTree<T>
where
    T: Select,
{
    fn select(&self, char: char, rank: usize) -> Option<usize> {
        let node = &self.root;
        let rank = rank;
        let interval_left = 0;
        let interval_right = self.alphabet.len() - 1;
        unsafe { self.select_inner(node, char, interval_left, interval_right, rank) }
    }
}

impl<T> WaveletTree<T>
where
    T: Select,
{
    /// # Safety
    /// This function is unsafe because it assumes that the left_interval and right_interval
    /// are valid indices in the alphabet vector. Calling this function with invalid indices
    /// will result in undefined behavior.
    #[inline]
    unsafe fn select_inner(
        &self,
        node: &WaveletTreeNode<T>,
        char: char,
        left_interval: usize,
        right_interval: usize,
        rank: usize,
    ) -> Option<usize> {
        match node {
            WaveletTreeNode::Leaf { len, .. } => {
                // here, we know that left_interval == right_interval
                // and that is the char that resides in the leaf node
                let leaf_char = self.alphabet.get_unchecked(left_interval);
                if char != *leaf_char {
                    return None;
                }

                if rank > *len {
                    return None;
                }
                Some(rank)
            }
            WaveletTreeNode::Internal {
                left,
                right,
                bit_vec,
            } => {
                let mid = (left_interval + right_interval) / 2;
                let mid_char = self.alphabet.get_unchecked(mid);
                if char <= *mid_char {
                    let index = self.select_inner(left, char, left_interval, mid, rank)?;
                    bit_vec.select0(index)
                } else {
                    let index = self.select_inner(right, char, mid + 1, right_interval, rank)?;
                    bit_vec.select(index)
                }
            }
        }
    }
}

impl<T> HeapSize for WaveletTree<T>
where
    T: HeapSize,
{
    fn heap_size_in_bits(&self) -> usize {
        self.alphabet.heap_size_in_bits() + self.root.heap_size_in_bits()
    }
}

impl<T> HeapSize for WaveletTreeNode<T>
where
    T: HeapSize,
{
    fn heap_size_in_bits(&self) -> usize {
        match self {
            Self::Internal {
                left,
                right,
                bit_vec,
            } => left.heap_size_in_bits() + right.heap_size_in_bits() + bit_vec.heap_size_in_bits(),
            Self::Leaf { len } => std::mem::size_of_val(len) * 8,
        }
    }
}

pub struct WaveletTreeSpec<B, T>
where
    B: Build<BitVec, T>,
{
    bit_vec_spec: B,
    phantom: PhantomData<T>,
}

impl<B, T> WaveletTreeSpec<B, T>
where
    B: Build<BitVec, T>,
{
    pub const fn new(bit_vec_spec: B) -> Self {
        Self {
            bit_vec_spec,
            phantom: PhantomData,
        }
    }
}

impl<T> WaveletTree<T> {
    #[inline]
    pub const fn spec<B>(spec: B) -> WaveletTreeSpec<B, T>
    where
        B: Build<BitVec, T>,
    {
        WaveletTreeSpec::new(spec)
    }
}

impl<B, T> Build<&str, WaveletTree<T>> for WaveletTreeSpec<B, T>
where
    B: Build<BitVec, T>,
{
    fn build(&self, data: &str) -> WaveletTree<T> {
        WaveletTree::new(data, &self.bit_vec_spec)
    }
}

impl<B, T> Build<&String, WaveletTree<T>> for WaveletTreeSpec<B, T>
where
    B: Build<BitVec, T>,
{
    fn build(&self, data: &String) -> WaveletTree<T> {
        WaveletTree::new(data, &self.bit_vec_spec)
    }
}

impl<B, T> Build<String, WaveletTree<T>> for WaveletTreeSpec<B, T>
where
    B: Build<BitVec, T>,
{
    fn build(&self, data: String) -> WaveletTree<T> {
        WaveletTree::new(&data, &self.bit_vec_spec)
    }
}

impl<B, T> Build<File, WaveletTree<T>> for WaveletTreeSpec<B, T>
where
    B: Build<BitVec, T>,
{
    fn build(&self, mut data: File) -> WaveletTree<T> {
        let raw_data = {
            let size = data.metadata().map(|m| m.len() as usize).ok();
            let mut bytes = Vec::with_capacity(size.unwrap_or(0));
            data.read_to_end(&mut bytes).unwrap();
            bytes
        };
        let data = String::from_utf8_lossy(&raw_data).to_string();
        WaveletTree::new(&data, &self.bit_vec_spec)
    }
}

#[cfg(test)]
mod tests;
