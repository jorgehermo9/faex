use super::VariableSizeIntVec;

impl VariableSizeIntVec<'_> {
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }
}
impl<'a> IntoIterator for VariableSizeIntVec<'a> {
    type Item = usize;
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<'a, 'b> IntoIterator for &'a VariableSizeIntVec<'b>
where
    'a: 'b,
{
    type Item = usize;
    type IntoIter = Iter<'a, 'b>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
pub struct IntoIter<'a> {
    vec: VariableSizeIntVec<'a>,
    index: usize,
}

impl<'a> IntoIter<'a> {
    pub fn new(vec: VariableSizeIntVec<'a>) -> Self {
        Self { vec, index: 0 }
    }
}
pub struct Iter<'a, 'b> {
    vec: &'a VariableSizeIntVec<'b>,
    index: usize,
}

impl<'a, 'b> Iter<'a, 'b>
where
    'a: 'b,
{
    pub fn new(vec: &'a VariableSizeIntVec<'b>) -> Self {
        Self { vec, index: 0 }
    }
}

impl Iterator for IntoIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.checked_get(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len();
        (len, Some(len))
    }
}

impl Iterator for Iter<'_, '_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.checked_get(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len();
        (len, Some(len))
    }
}
