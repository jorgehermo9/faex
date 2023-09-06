pub mod wavelet_tree;

/// Trait for data structures that can support rank queries over a sequence of characters. (typically, wavelet trees)
pub trait CharacterRank {
    /// Returns the number of occurrences of `char` in the sequence up to (excluding) position `index`.
    /// i.e in range [0, i)
    fn rank(&self, char: char, index: usize) -> Option<usize>;
}

pub trait CharacterSelect {
    /// Returns the position that leaves `index` occurrences of `cchar` in the sequence behind that position (excluded).
    fn select(&self, char: char, index: usize) -> Option<usize>;
}

pub trait CharacterAccess {
    /// Returns the character at position `index` in the sequence.
    fn access(&self, index: usize) -> Option<char>;
}
