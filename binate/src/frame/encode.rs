use bytes::{Bytes, BytesMut};

/// A trait for encoding a frame into bytes.
pub trait Encode {
    /// Encodes `self` into bytes.
    fn encode(&self, buf: &mut BytesMut);

    /// Returns the length (in bytes) of this value.
    fn len(&self) -> usize;

    /// Returns whether this value is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Encodes `self` into bytes.
    ///
    /// This function is guaranteed to return a `Bytes` whose capacity is exactly the same as
    /// `self.len()`.
    fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.len());
        self.encode(&mut buf);
        buf.freeze()
    }
}
