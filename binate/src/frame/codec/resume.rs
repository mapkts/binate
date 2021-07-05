use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// The resume frame.
///
/// # Frame Contents
///
/// The general format for a Resume frame is given below.
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Stream ID = 0                         |
/// +-----------+-+-+---------------+-------------------------------+
/// |Frame Type |0|0|    Flags      |
/// +-------------------------------+-------------------------------+
/// |        Major Version          |         Minor Version         |
/// +-------------------------------+-------------------------------+
/// |         Token Length          | Resume Identification Token  ...
/// +---------------------------------------------------------------+
/// |0|                                                             |
/// +                 Last Received Server Position                 +
/// |                                                               |
/// +---------------------------------------------------------------+
/// |0|                                                             |
/// +                First Available Client Position                +
/// |                                                               |
/// +---------------------------------------------------------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeFrame {
    version: Version,
    resume_token: Bytes,
    last_received_server_position: u64,
    first_available_client_position: u64,
}

impl ResumeFrame {
    /// RESUME frames MUST always use Stream ID 0 as they pertain to the connection.
    pub const STREAM_ID: u32 = 0;

    /// Create a new `Resume` frame.
    ///
    /// - The length of `resume_token` MUST be <= `65,535` bytes long.
    /// - Both `last_received_server_position` and `first_available_client_position` MUST be <=
    /// [`MAX_U63`].
    pub fn new(
        version: Version,
        resume_token: Bytes,
        mut last_received_server_position: u64,
        mut first_available_client_position: u64,
    ) -> Self {
        debug_assert_max_u63!(
            last_received_server_position,
            first_available_client_position
        );
        last_received_server_position &= MAX_U63;
        first_available_client_position &= MAX_U63;

        ResumeFrame {
            version,
            resume_token,
            last_received_server_position,
            first_available_client_position,
        }
    }

    /// Returns the version field of this frame.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the resume identification token.
    pub fn resume_token(&self) -> &Bytes {
        &self.resume_token
    }

    /// Returns the last implied position the client received from the server.
    pub fn last_received_server_position(&self) -> u64 {
        self.last_received_server_position
    }

    /// Returns the earliest position that the client can rewind back to prior to resending frames.
    pub fn first_available_client_position(&self) -> u64 {
        self.first_available_client_position
    }
}

impl Encode for ResumeFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(0);
        buf.put_u16(FrameType::RESUME.bits());
        self.version.encode(buf);
        buf.put_u16(self.resume_token.len() as u16);
        buf.put_slice(&self.resume_token);
        buf.put_u64(self.last_received_server_position);
        buf.put_u64(self.first_available_client_position);
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        // len(version): 4
        // len(token_length): 2
        // len(resume_token)
        // len(last_received_server_position): 8
        // len(first_available_client_position): 8
        12 + self.resume_token.len() + 16
    }
}

impl Decode for ResumeFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        _stream_id: u32,
        _flags: Flags,
    ) -> Result<Self::Value> {
        let version = eat_version(buf)?;
        let token_len = eat_u16(buf)?;
        let resume_token = eat_bytes(buf, token_len as usize)?;
        let last_received_server_position = eat_u63(buf)?;
        let first_available_client_position = eat_u63(buf)?;
        Ok(ResumeFrame {
            version,
            resume_token,
            last_received_server_position,
            first_available_client_position,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let lease = ResumeFrame::new(
            Version::new(1, 0),
            Bytes::from("resume token"),
            1,
            2,
        );

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(version): 4
        // len(token_length): 2
        // len(resume_token): 12
        // len(last_received_server_position): 8
        // len(first_available_client_position): 8
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2 + 4 + 2 + 12 + 8 + 8);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(frame_type, FrameType::RESUME);
        assert_eq!(flags, Flags::empty());

        let decoded = ResumeFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
