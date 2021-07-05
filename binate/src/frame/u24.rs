use super::Encode;
use bytes::{BufMut, BytesMut};

/// The 24-bit unsigned integer type.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct U24(pub(crate) u8, pub(crate) u16);

impl U24 {
    /// The maximum value `U24` can hold.
    pub const MAX: u32 = u32::MAX >> 8;

    /// The minimum value `U24` can hold.
    pub const MIN: u32 = 0;

    /// Builds a `U24` from the given `higher_bits` and `lower_bits`.
    pub fn new(higher_bits: u8, lower_bits: u16) -> Self {
        U24(higher_bits, lower_bits)
    }

    /// Builds a `U24` from a `u32` value.
    ///
    /// # Panics
    ///
    /// Panics if the value given is greater than `U24::MAX` (max value 16,777,215).
    pub fn from_u32(val: u32) -> Self {
        assert!(val <= U24::MAX);
        U24((val >> 16) as u8, val as u16)
    }

    /// Builds a `U24` from a `usize` value.
    ///
    /// # Panics
    ///
    /// Panics if the value given is greater than `U24::MAX` (max value 16,777,215).
    pub fn from_usize(val: usize) -> Self {
        assert!(val <= U24::MAX as usize);
        U24((val as u32 >> 16) as u8, (val as u32) as u16)
    }

    /// Converts `Self` into `u32`.
    pub fn into_u32(self) -> u32 {
        ((self.0 as u32) << 16) | (self.1 as u32)
    }

    /// Converts `Self` into `usize`.
    pub fn into_usize(self) -> usize {
        ((self.0 as usize) << 16) | (self.1 as usize)
    }
}

impl Encode for U24 {
    fn len(&self) -> usize {
        3
    }

    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(self.0);
        buf.put_u16(self.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u24_min() {
        assert_eq!(U24::MIN, 0);
    }

    #[test]
    fn test_u24_max() {
        assert_eq!(U24::MAX, 16_777_215);
    }

    #[test]
    fn test_from_to_u32() {
        assert_eq!(U24::from_u32(U24::MAX).into_u32(), U24::MAX);
    }

    #[test]
    fn test_from_to_usize() {
        assert_eq!(
            U24::from_usize(U24::MAX as usize).into_usize(),
            U24::MAX as usize
        );
    }

    #[test]
    #[should_panic]
    fn test_from_invalid_u32() {
        U24::from_u32(U24::MAX as u32 + 1);
    }

    #[test]
    #[should_panic]
    fn test_from_invalid_usize() {
        U24::from_usize(U24::MAX as usize + 1);
    }
}
