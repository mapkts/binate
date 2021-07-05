use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// The payload frame.
///
/// # Frame Contents
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Stream ID                           |
/// +-----------+-+-+-+-+-+---------+-------------------------------+
/// |Frame Type |0|M|F|C|N|  Flags  |
/// +-------------------------------+-------------------------------+
///                      Metadata & Data
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadFrame {
    stream_id: u32,
    flags: Flags,
    payload: Payload,
}

impl PayloadFrame {
    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::PAYLOAD;

    /// Create a new `Payload` frame.
    ///
    /// - `stream_id` MUST be <= [`MAX_U31`].
    /// - flag `follows` means more fragments follow this fragment.
    /// - flag `complete` indicates stream completion. If set, `on_complete()` will be invoked on
    /// Subscriber/Observer.
    /// - flag `next` indicates Next (Payload Data and/or Metadata present). If set,
    /// `on_next(Payload)` will be invoked on Subscriber/Observer.
    ///
    /// A PAYLOAD MUST NOT have both (C)complete and (N)ext empty (false). See [`Payload Frame`]
    /// section in the spec for more details.
    ///
    /// [`Payload Frame`]: https://rsocket.io/about/protocol/#payload-frame-0x0a
    pub fn new(stream_id: u32, mut flags: Flags, payload: Payload) -> Self {
        debug_assert_max_u31!(stream_id);
        let stream_id = stream_id & MAX_U31;
        flags &= Flags::FOLLOWS | Flags::COMPLETE | Flags::NEXT;
        if payload.has_metadata() {
            flags |= Flags::METADATA
        }
        PayloadFrame { stream_id, flags, payload }
    }

    /// Returns the stream ID of this frame.
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }

    /// Returns true if this frame has the FOLLOWS flag set.
    pub fn is_follows(&self) -> bool {
        self.flags.contains(Flags::FOLLOWS)
    }

    /// Returns true if this frame has the COMPLETE flag set.
    pub fn is_complete(&self) -> bool {
        self.flags.contains(Flags::COMPLETE)
    }

    /// Returns true if this frame has the NEXT flag set.
    pub fn is_next(&self) -> bool {
        self.flags.contains(Flags::NEXT)
    }

    /// Returns the metadata attached to this frame, if any.
    pub fn metadata(&self) -> Option<&Bytes> {
        self.payload.metadata()
    }

    /// Returns the data attached to this frame, if any.
    pub fn data(&self) -> Option<&Bytes> {
        self.payload.data()
    }

    /// Returns the payload attached to this frame.
    pub fn payload(self) -> Payload {
        self.payload
    }
}

impl Encode for PayloadFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(self.stream_id);
        buf.put_u16(FrameType::PAYLOAD.bits() | self.flags.bits());
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

impl Decode for PayloadFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        stream_id: u32,
        flags: Flags,
    ) -> Result<Self::Value> {
        let payload = eat_payload(buf, true)?;
        Ok(PayloadFrame { stream_id, flags, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let lease = PayloadFrame::new(
            1,
            Flags::FOLLOWS | Flags::NEXT,
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
        assert_eq!(frame_type, FrameType::PAYLOAD);
        assert_eq!(flags, Flags::METADATA | Flags::FOLLOWS | Flags::NEXT);

        let decoded =
            PayloadFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
