// Bitfield  macros

/// The mask to extract $len bits at bit offset $off.
// generates a bitmask of max len 64

pub(crate) const USIZE_BITS: usize = std::mem::size_of::<usize>() * 8;

/// The mask to extract $len bits at bit offset $off. Only valid for usize values.
macro_rules! bitmask {
    ( $len:expr, $off:expr ) => {
        usize::MAX
            .checked_shr((crate::util::USIZE_BITS - $len) as u32)
            .unwrap_or(0)
            << $off
    };
}

/// Extract $len bits from $val at bit offset $off. Only valid for usize values.
#[macro_export]
macro_rules! getbits {
    ( $val: expr, $len:expr, $off:expr ) => {
        ($val & $crate::util::bitmask!($len, $off)) >> $off
    };
}

/// Update $len bits in $var at bit offset $off to $val. Only valid for usize values.
#[macro_export]
macro_rules! setbits {
    ( $var: expr, $len:expr, $off:expr, $val: expr ) => {
        $var = ($var & !$crate::util::bitmask!($len, $off))
            | (($val << $off) & $crate::util::bitmask!($len, $off))
    };
}

/// Required bits in order to represent a value.
/// For example, value 0 requires 0 bits, 1 requires 1 bit, 2 requires 2 bits, 3 requires 2 bits, etc.
pub trait BitsRequired {
    fn bits_required(&self) -> u32;
}

macro_rules! impl_bits_required_for {
    ($($t:ty),*) => {
        $(

            impl BitsRequired for $t {
                fn bits_required(&self) -> u32 {
                    self.checked_ilog2().map(|v| v + 1).unwrap_or(0)
                }
            }

        )*
    };
}
impl_bits_required_for!(u8, u16, usize, u32, u64, u128);

pub(crate) fn ceil_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

pub(crate) use bitmask;
pub(crate) use getbits;
pub(crate) use setbits;
