use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// The meatadata_push frame.
///
/// A Metadata Push frame can be used to send asynchronous metadata notifications from a Requester
/// or Responder to its peer.
///
/// # Frame Contents
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Stream ID = 0                         |
/// +-----------+-+-+---------------+-------------------------------+
/// |Frame Type |0|1|     Flags     |
/// +-------------------------------+-------------------------------+
///                             Metadata
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataPushFrame {
    metadata: Bytes,
}

impl MetadataPushFrame {
    /// METADATA_PUSH frames MUST always use Stream ID 0 as they pertain to the Connection.
    pub const STREAM_ID: u32 = 0;

    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::METADATA_PUSH;

    /// Create a `MetadataPush` frame.
    pub fn new(metadata: Bytes) -> Self {
        MetadataPushFrame { metadata }
    }

    /// Returns the metadata attached to this frame, if any.
    pub fn metadata(&self) -> &Bytes {
        &self.metadata
    }
}

impl Encode for MetadataPushFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(0);
        buf.put_u16(FrameType::METADATA_PUSH.bits() | Flags::METADATA.bits());
        buf.put_slice(&self.metadata);
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        // len(metadata)
        6 + self.metadata.len()
    }
}

impl Decode for MetadataPushFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        _stream_id: u32,
        _flags: Flags,
    ) -> Result<Self::Value> {
        let metadata = eat_bytes(buf, buf.remaining())?;
        Ok(MetadataPushFrame { metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let lease = MetadataPushFrame::new(Bytes::from("metadata"));

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(metadata): 8
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2 + 8);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(frame_type, FrameType::METADATA_PUSH);
        assert_eq!(flags, Flags::METADATA);

        let decoded =
            MetadataPushFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
