use super::*;
use bytes::{Buf, BufMut, BytesMut};

/// The cancel frame.
///
/// # Frame Contents
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Stream ID                           |
/// +-----------+-+-+---------------+-------------------------------+
/// |Frame Type |0|0|    Flags      |
/// +-------------------------------+-------------------------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelFrame {
    stream_id: u32,
}

impl CancelFrame {
    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::CANCEL;

    /// Create a new `Cancel` frame.
    ///
    /// - `stream_id` MUST be <= [`MAX_U31`].
    pub fn new(stream_id: u32) -> Self {
        debug_assert_max_u31!(stream_id);
        let stream_id = stream_id & MAX_U31;
        CancelFrame { stream_id }
    }

    /// Returns the stream ID of this frame.
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }
}

impl Encode for CancelFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(self.stream_id);
        buf.put_u16(FrameType::CANCEL.bits());
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        6
    }
}

impl Decode for CancelFrame {
    type Value = Self;

    fn decode<B: Buf>(
        _buf: &mut B,
        stream_id: u32,
        _flags: Flags,
    ) -> Result<Self::Value> {
        Ok(CancelFrame { stream_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let lease = CancelFrame::new(1);

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(frame_type, FrameType::CANCEL);
        assert_eq!(flags, Flags::empty());

        let decoded = CancelFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
