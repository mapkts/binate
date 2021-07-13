use super::*;
use bytes::{Buf, BufMut, BytesMut};

/// The request_n frame.
///
/// # Frame Contents
///
/// The request_n frame is structured as follows:
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Stream ID                           |
/// +-----------+-+-+---------------+-------------------------------+
/// |Frame Type |0|0|     Flags     |
/// +-------------------------------+-------------------------------+
/// |0|                         Request N                           |
/// +---------------------------------------------------------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestNFrame {
    stream_id: u32,
    request_n: u32,
}

impl RequestNFrame {
    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::REQUEST_N;

    /// Create a new `RequestN` frame.
    ///
    /// - `stream_id` MUST be <= [`MAX_U31`].
    /// - `request_n` represents the number of items to request. Value MUST be > 0 and
    /// <= [`MAX_U31`].
    pub fn new(stream_id: u32, request_n: u32) -> Self {
        debug_assert_max_u31!(stream_id, request_n);
        debug_assert_non_zero!(request_n);
        let stream_id = stream_id & MAX_U31;
        let request_n = request_n & MAX_U31;
        RequestNFrame { stream_id, request_n }
    }

    /// Returns the stream ID of this frame.
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }

    /// Returns the number of items to request.
    pub fn request_n(&self) -> u32 {
        self.request_n
    }
}

impl Encode for RequestNFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(self.stream_id);
        buf.put_u16(FrameType::REQUEST_N.bits());
        buf.put_u32(self.request_n);
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        // len(request_n): 4
        10
    }
}

impl Decode for RequestNFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        stream_id: u32,
        _flags: Flags,
    ) -> Result<Self::Value> {
        let request_n = eat_u31(buf)?;
        Ok(RequestNFrame { stream_id, request_n })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let lease = RequestNFrame::new(1, 2);

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(request_n): 4
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2 + 4);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(frame_type, FrameType::REQUEST_N);
        assert_eq!(flags, Flags::empty());

        let decoded =
            RequestNFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
