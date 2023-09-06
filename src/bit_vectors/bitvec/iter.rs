use super::BitVec;

impl BitVec {
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }
}

impl IntoIterator for BitVec {
    type Item = bool;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a BitVec {
    type Item = bool;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntoIter {
    bitvec: BitVec,
    index: usize,
}
impl IntoIter {
    pub fn new(bitvec: BitVec) -> Self {
        Self { bitvec, index: 0 }
    }
}
pub struct Iter<'a> {
    bitvec: &'a BitVec,
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new(bitvec: &'a BitVec) -> Self {
        Self { bitvec, index: 0 }
    }
}

impl Iterator for IntoIter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.bitvec.len {
            return None;
        }
        let val = self.bitvec.read(self.index);
        self.index += 1;
        Some(val)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.bitvec.len();
        (len, Some(len))
    }
}

impl Iterator for Iter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.bitvec.len {
            return None;
        }
        let val = self.bitvec.read(self.index);
        self.index += 1;
        Some(val)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.bitvec.len();
        (len, Some(len))
    }
}

// Create Bitvec from other iterable types

impl From<&[bool]> for BitVec {
    fn from(slice: &[bool]) -> Self {
        let mut bitvec = BitVec::with_capacity(slice.len());
        for bit in slice {
            bitvec.push(*bit);
        }
        bitvec
    }
}

// TODO: collect bools,u8 and u16 into usize and then push?
// using slice::chunks method
impl From<Vec<bool>> for BitVec {
    fn from(vec: Vec<bool>) -> Self {
        vec.as_slice().into()
    }
}

impl<const N: usize> From<[bool; N]> for BitVec {
    fn from(array: [bool; N]) -> Self {
        array.as_ref().into()
    }
}

impl FromIterator<bool> for BitVec {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let mut bitvec = BitVec::new();
        for bit in iter {
            bitvec.push(bit);
        }
        bitvec
    }
}

impl Extend<bool> for BitVec {
    fn extend<T: IntoIterator<Item = bool>>(&mut self, iter: T) {
        for bit in iter {
            self.push(bit);
        }
    }
}

impl<'a> Extend<&'a bool> for BitVec {
    fn extend<T: IntoIterator<Item = &'a bool>>(&mut self, iter: T) {
        for bit in iter {
            self.push(*bit);
        }
    }
}

macro_rules! impl_from_for {
    ($($t:ty),*) => {
        $(
            impl From<&[$t]> for BitVec {
                fn from(slice: &[$t]) -> Self {
                    let mut bitvec = BitVec::with_capacity(std::mem::size_of_val(slice) * 8);
                    for value in slice {
                        bitvec.push_bits(*value, std::mem::size_of::<$t>() * 8);
                    }
                    bitvec
                }
            }

            impl From<Vec<$t>> for BitVec {
                fn from(vec: Vec<$t>) -> Self {
                    vec.as_slice().into()
                }
            }

            impl<const N: usize> From<[$t; N]> for BitVec {
                fn from(array: [$t; N]) -> Self {
                    array.as_ref().into()
                }
            }
            impl<const N: usize> From<&[$t; N]> for BitVec {
                fn from(array: &[$t; N]) -> Self {
                    array.as_ref().into()
                }
            }

            impl FromIterator<$t> for BitVec {
                fn from_iter<I: IntoIterator<Item = $t>>(iter: I) -> Self {
                    let mut bitvec = BitVec::new();
                    for value in iter {
                        bitvec.push_bits(value, std::mem::size_of::<$t>() * 8);
                    }
                    bitvec
                }
            }

            impl Extend<$t> for BitVec {
                fn extend<I: IntoIterator<Item = $t>>(&mut self, iter: I) {
                    for value in iter {
                        self.push_bits(value, std::mem::size_of::<$t>() * 8);
                    }
                }
            }

            impl<'a> Extend<&'a $t> for BitVec {
                fn extend<I: IntoIterator<Item = &'a $t>>(&mut self, iter: I) {
                    for value in iter {
                        self.push_bits(*value, std::mem::size_of::<$t>() * 8);
                    }
                }
            }
        )*
    };
}
impl_from_for!(u8, u16, usize);
