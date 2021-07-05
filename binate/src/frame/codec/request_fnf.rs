use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// The request_fnf (Fire-n-Forget) frame.
///
/// # Frame Contents
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Stream ID                           |
/// +-----------+-+-+-+-------------+-------------------------------+
/// |Frame Type |0|M|F|     Flags   |
/// +-------------------------------+
///                      Metadata & Request Data
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestFnfFrame {
    stream_id: u32,
    flags: Flags,
    payload: Payload,
}

impl RequestFnfFrame {
    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::REQUEST_FNF;

    /// Create a new `RequestFnf` frame.
    ///
    /// - `stream_id` MUST be <= [`MAX_U31`].
    /// - flag `follows` means more fragments follow this fragment.
    pub fn new(stream_id: u32, follows: bool, payload: Payload) -> Self {
        debug_assert_max_u31!(stream_id);
        let stream_id = stream_id & MAX_U31;
        let mut flags = Flags::empty();
        if follows {
            flags |= Flags::FOLLOWS
        }
        if payload.has_metadata() {
            flags |= Flags::METADATA
        }
        RequestFnfFrame { stream_id, flags, payload }
    }

    /// Returns the stream ID of this frame.
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }

    /// Returns true if this frame has the FOLLOWS flag set.
    pub fn is_follows(&self) -> bool {
        self.flags.contains(Flags::FOLLOWS)
    }

    /// Returns the metadata attached to this frame, if any.
    pub fn metadata(&self) -> Option<&Bytes> {
        self.payload.metadata()
    }

    /// Returns the request data attached to this frame, if any.
    pub fn request_data(&self) -> Option<&Bytes> {
        self.payload.data()
    }

    /// Returns the payload attached to this frame.
    pub fn payload(self) -> Payload {
        self.payload
    }
}

impl Encode for RequestFnfFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(self.stream_id);
        buf.put_u16(FrameType::REQUEST_FNF.bits() | self.flags.bits());
        let u24 = U24::from_usize(
            self.payload.metadata().map(|v| v.len()).unwrap_or_default(),
        );
        buf.put_u8(u24.0);
        buf.put_u16(u24.1);
        self.payload.encode(buf);
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        // len(metadata_len): 3
        // len(payload)
        9 + self.payload.len()
    }
}

impl Decode for RequestFnfFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        stream_id: u32,
        flags: Flags,
    ) -> Result<Self::Value> {
        let payload = eat_payload(buf, true)?;
        Ok(RequestFnfFrame { stream_id, flags, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let lease = RequestFnfFrame::new(
            1,
            true,
            Payload::builder()
                .set_metadata(Bytes::from("metadata"))
                .set_data(Bytes::from("data"))
                .build(),
        );

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(metadata_len): 3
        // len(metadata): 8
        // len(data): 4
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2 + 3 + 8 + 4);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(frame_type, FrameType::REQUEST_FNF);
        assert_eq!(flags, Flags::METADATA | Flags::FOLLOWS);

        let decoded =
            RequestFnfFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
