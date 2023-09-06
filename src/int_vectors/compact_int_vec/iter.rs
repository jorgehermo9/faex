use crate::util::BitsRequired;

use super::CompactIntVec;

impl CompactIntVec {
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }
}
impl IntoIterator for CompactIntVec {
    type Item = usize;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a CompactIntVec {
    type Item = usize;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
pub struct IntoIter {
    compact_int_vec: CompactIntVec,
    index: usize,
}

impl IntoIter {
    pub fn new(compact_int_vec: CompactIntVec) -> Self {
        Self {
            compact_int_vec,
            index: 0,
        }
    }
}
pub struct Iter<'a> {
    compact_int_vec: &'a CompactIntVec,
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new(compact_int_vec: &'a CompactIntVec) -> Self {
        Self {
            compact_int_vec,
            index: 0,
        }
    }
}

impl Iterator for IntoIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.compact_int_vec.get(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.compact_int_vec.len();
        (len, Some(len))
    }
}

impl Iterator for Iter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.compact_int_vec.get(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.compact_int_vec.len();
        (len, Some(len))
    }
}

/// Trait to calculate the number of bits required to represent a value

impl<T> From<&[T]> for CompactIntVec
where
    T: BitsRequired + Copy + Into<usize> + Ord,
{
    fn from(slice: &[T]) -> Self {
        assert!(
            !slice.is_empty(),
            "Cannot create a CompactIntVec from an empty slice, width cannot be inferred"
        );

        let max_value = slice.iter().max().unwrap();
        let bitwidth = max_value.bits_required() as usize;
        let mut compact_int_vec = CompactIntVec::new(bitwidth);
        for value in slice {
            compact_int_vec.push(*value);
        }
        compact_int_vec
    }
}

/// CompactIntVec's int width is inferred from the Vec
impl<T> From<Vec<T>> for CompactIntVec
where
    T: BitsRequired + Copy + Into<usize> + Ord,
{
    fn from(vec: Vec<T>) -> Self {
        assert!(
            !vec.is_empty(),
            "Cannot create a CompactIntVec from an empty Vec, width cannot be inferred"
        );

        Self::from(&vec[..])
    }
}

/// CompactIntVec's int width is inferred from the array
impl<T, const N: usize> From<[T; N]> for CompactIntVec
where
    T: BitsRequired + Copy + Into<usize> + Ord,
{
    fn from(array: [T; N]) -> Self {
        assert!(
            N > 0,
            "Cannot create a CompactIntVec from an empty array, width cannot be inferred"
        );

        Self::from(&array[..])
    }
}

impl<T, const N: usize> From<&[T; N]> for CompactIntVec
where
    T: BitsRequired + Copy + Into<usize> + Ord,
{
    fn from(array: &[T; N]) -> Self {
        assert!(
            N > 0,
            "Cannot create a CompactIntVec from an empty array, width cannot be inferred"
        );

        Self::from(&array[..])
    }
}

/// CompactIntVec's int width is inferred from the iterator.
/// This is a costly transformation, since it needs to collect all the elements into a Vec to calculate the bitwidth
impl<T> FromIterator<T> for CompactIntVec
where
    T: BitsRequired + Copy + Into<usize> + Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        // TODO: avoid collecting into a Vec, and calculate the bitwidth from the iterator??
        let items = iter.into_iter().collect::<Vec<T>>();
        assert!(
            !items.is_empty(),
            "Cannot create a CompactIntVec from an empty iterator, width cannot be inferred"
        );
        Self::from(&items[..])
    }
}

impl<T> Extend<T> for CompactIntVec
where
    T: Into<usize>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}
