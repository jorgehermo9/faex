pub mod bit_vectors;
pub mod character_sequence;
pub mod int_vectors;
pub mod profiling;
pub mod util;

pub trait Build<T, O> {
    fn build(&self, data: T) -> O;
}
