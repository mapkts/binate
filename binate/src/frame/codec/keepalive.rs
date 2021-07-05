use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// The keepalive frame.
///
/// # Frame Contents
///
/// ```text
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Stream ID = 0                         |
/// +-----------+-+-+-+-------------+-------------------------------+
/// |Frame Type |0|0|R|    Flags    |
/// +-----------+-+-+-+-------------+-------------------------------+
/// |0|                  Last Received Position                     |
/// +                                                               +
/// |                                                               |
/// +---------------------------------------------------------------+
///                              Data
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeepaliveFrame {
    respond: bool,
    last_received_position: u64,
    data: Option<Bytes>,
}

impl KeepaliveFrame {
    /// KEEPALIVE frames MUST always use Stream ID 0 as they pertain to the Connection.
    pub const STREAM_ID: u32 = 0;

    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::KEEPALIVE;

    /// Create a new `Keepalive` frame.
    ///
    /// - `last_received_position` MUST be <= [`MAX_U63`].
    pub fn new(
        last_received_position: u64,
        data: Option<Bytes>,
        respond: bool,
    ) -> Self {
        debug_assert_max_u63!(last_received_position);
        KeepaliveFrame {
            respond,
            last_received_position: last_received_position & MAX_U63,
            data,
        }
    }

    /// Returns the last received position of this frame.
    pub fn last_received_position(&self) -> u64 {
        self.last_received_position
    }

    /// Returns the data attached to this frame, if any.
    pub fn data(&self) -> Option<&Bytes> {
        self.data.as_ref()
    }

    /// Returns true if this frame has the Respond flag set.
    pub fn is_respond(&self) -> bool {
        self.respond
    }
}

impl Encode for KeepaliveFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(0);
        if self.respond {
            buf.put_u16(FrameType::KEEPALIVE.bits() | Flags::RESPOND.bits());
        } else {
            buf.put_u16(FrameType::KEEPALIVE.bits());
        }
        buf.put_u64(self.last_received_position);
        if let Some(data) = &self.data {
            buf.put_slice(data);
        }
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        // len(last_received_position): 8
        let mut len = 14;

        // len(data)
        if let Some(data) = &self.data {
            len += data.len();
        }
        len
    }
}

impl Decode for KeepaliveFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        stream_id: u32,
        flags: Flags,
    ) -> Result<Self::Value> {
        if stream_id != 0 {
            return Err(DecodeError::InvalidStreamId {
                expected: "0",
                found: stream_id,
            });
        }
        let respond = flags.contains(Flags::RESPOND);
        let last_received_position = eat_u63(buf)?;
        let data = match buf.remaining() {
            0 => None,
            len => Some(eat_bytes(buf, len)?),
        };
        Ok(KeepaliveFrame { respond, last_received_position, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_id() {
        assert_eq!(KeepaliveFrame::STREAM_ID, 0);
    }

    #[test]
    fn test_codec() {
        let lease = KeepaliveFrame::new(1, Some(Bytes::from("data")), true);

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(last_received_position): 8
        // len(data): 4
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2 + 8 + 4);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(stream_id, 0);
        assert_eq!(frame_type, FrameType::KEEPALIVE);
        assert_eq!(flags, Flags::RESPOND);

        let decoded =
            KeepaliveFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
