use crate::util::ceil_div;

// TODO: see https://github.com/dtolnay/syn/tree/master/examples/heapsize
pub trait HeapSize {
    /// Returns the size of the data structure in bits.
    fn heap_size_in_bits(&self) -> usize;

    /// Returns the size of the data structure in bytes.
    fn heap_size_in_bytes(&self) -> usize {
        // Round-up to the nearest byte
        ceil_div(self.heap_size_in_bits(), 8)
    }

    /// Returns the size of the data structure in kibibytes.
    fn heap_size_in_kib(&self) -> usize {
        ceil_div(self.heap_size_in_bytes(), 1024)
    }

    /// Returns the size of the data structure in mebibytes.
    fn heap_size_in_mib(&self) -> usize {
        ceil_div(self.heap_size_in_kib(), 1024)
    }
    /// Returns the size of the data structure in gibibytes.
    fn heap_size_in_gib(&self) -> usize {
        ceil_div(self.heap_size_in_mib(), 1024)
    }

    /// Returns the exact size of the data structure in bytes as a floating point number.
    fn exact_heap_size_in_bytes(&self) -> f64 {
        self.heap_size_in_bits() as f64 / 8.0
    }

    /// Returns the exact size of the data structure in kibibytes as a floating point number.
    fn exact_heap_size_in_kib(&self) -> f64 {
        self.exact_heap_size_in_bytes() / 1024.0
    }

    /// Returns the exact size of the data structure in mebibytes as a floating point number.
    fn exact_heap_size_in_mib(&self) -> f64 {
        self.exact_heap_size_in_kib() / 1024.0
    }

    /// Returns the exact size of the data structure in gibibytes as a floating point number.
    fn exact_heap_size_in_gib(&self) -> f64 {
        self.exact_heap_size_in_mib() / 1024.0
    }
}

impl<T> HeapSize for &[T] {
    fn heap_size_in_bits(&self) -> usize {
        std::mem::size_of_val(*self) * 8
    }
}
impl<T> HeapSize for Vec<T> {
    fn heap_size_in_bits(&self) -> usize {
        std::mem::size_of::<T>() * 8 * self.len()
    }
}
