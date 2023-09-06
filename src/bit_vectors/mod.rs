pub mod bitvec;
pub mod rank_select;
pub mod rrr_bitvec;

// TODO: not implemented yet.
// pub mod sd_vec;

pub use crate::bit_vectors::bitvec::BitVec;
pub use crate::bit_vectors::rrr_bitvec::RRRBitVec;

pub trait Access {
    // Trait that defines access operation.
    fn access(&self, index: usize) -> Option<bool>;
}
