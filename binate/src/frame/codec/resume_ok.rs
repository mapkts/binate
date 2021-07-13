use super::*;
use bytes::{Buf, BufMut, BytesMut};

/// The resume_ok frame.
///
/// # Frame Contents
///
/// The resume_ok frame is structured as follows:
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Stream ID = 0                         |
/// +-----------+-+-+---------------+-------------------------------+
/// |Frame Type |0|0|    Flags      |
/// +-------------------------------+-------------------------------+
/// |0|                                                             |
/// +               Last Received Client Position                   +
/// |                                                               |
/// +---------------------------------------------------------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeOkFrame {
    last_received_client_position: u64,
}

impl ResumeOkFrame {
    /// RESUME OK frames MUST always use Stream ID 0 as they pertain to the connection.
    pub const STREAM_ID: u32 = 0;

    /// Create a new `ResumeOk` frame.
    ///
    /// - `last_received_client_position` and `first_available_client_position` MUST be <=
    /// [`MAX_U63`].
    pub fn new(mut last_received_client_position: u64) -> Self {
        debug_assert_max_u63!(last_received_client_position);
        last_received_client_position &= MAX_U63;
        ResumeOkFrame { last_received_client_position }
    }

    /// Returns the last implied position the server received from the client.
    pub fn last_received_server_position(&self) -> u64 {
        self.last_received_client_position
    }
}

impl Encode for ResumeOkFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(0);
        buf.put_u16(FrameType::RESUME_OK.bits());
        buf.put_u64(self.last_received_client_position);
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        // len(last_received_client_position): 8
        14
    }
}

impl Decode for ResumeOkFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        _stream_id: u32,
        _flags: Flags,
    ) -> Result<Self::Value> {
        let last_received_client_position = eat_u63(buf)?;
        Ok(ResumeOkFrame { last_received_client_position })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let lease = ResumeOkFrame::new(1);

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(last_received_client_position): 8
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2 + 8);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(frame_type, FrameType::RESUME_OK);
        assert_eq!(flags, Flags::empty());

        let decoded =
            ResumeOkFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
