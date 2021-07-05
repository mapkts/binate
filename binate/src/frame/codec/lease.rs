use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::Duration;

/// The lease frame.
///
/// Lease frames MAY be sent by the client-side or server-side Responders and inform the Requester
/// that it may send Requests for a period of time and how many it may send during that duration.
///
/// # Frame Contents
///
/// ```text
/// 0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Stream ID = 0                         |
/// +-----------+-+-+---------------+-------------------------------+
/// |Frame Type |0|M|     Flags     |
/// +-----------+-+-+---------------+-------------------------------+
/// |0|                       Time-To-Live                          |
/// +---------------------------------------------------------------+
/// |0|                     Number of Requests                      |
/// +---------------------------------------------------------------+
///                             Metadata
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseFrame {
    ttl: u32,
    number_of_requests: u32,
    metadata: Option<Bytes>,
}

impl LeaseFrame {
    /// Lease frames MUST always use Stream ID 0 as they pertain to the connection.
    pub const STREAM_ID: u32 = 0;

    /// Create a new `Lease` frame.
    ///
    /// - The `ttl` (Time to Live) is measured in milliseconds.
    /// - Both `ttl` and `number_of_requests` MUST be <= [`MAX_U31`].
    pub fn new(
        ttl: u32,
        number_of_requests: u32,
        metadata: Option<Bytes>,
    ) -> Self {
        debug_assert_max_u31!(ttl, number_of_requests);
        LeaseFrame { ttl, number_of_requests, metadata }
    }

    /// Returns the validity time (in milliseconds) of LEASE from time of reception.
    pub fn ttl(&self) -> Duration {
        Duration::from_millis(self.ttl as u64)
    }

    /// Returns the number of requests that may be sent until next LEASE.
    pub fn number_of_requests(&self) -> u32 {
        self.number_of_requests
    }

    /// Returns the metadata attached to this frame, if any.
    pub fn metadata(&self) -> Option<&Bytes> {
        self.metadata.as_ref()
    }
}

impl Encode for LeaseFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(0);
        if self.metadata().is_some() {
            buf.put_u16(FrameType::LEASE.bits() | Flags::METADATA.bits());
        } else {
            buf.put_u16(FrameType::LEASE.bits());
        };
        buf.put_u32(self.ttl);
        buf.put_u32(self.number_of_requests);
        if let Some(metadata) = &self.metadata {
            buf.put_slice(metadata);
        }
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(flags): 2
        // len(ttl): 4
        // len(number_of_requests): 4
        let mut len = 14;

        // len(metadata)
        if let Some(metadata) = &self.metadata {
            len += metadata.len();
        }
        len
    }
}

impl Decode for LeaseFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        stream_id: u32,
        _flags: Flags,
    ) -> Result<Self::Value> {
        if stream_id != 0 {
            return Err(DecodeError::InvalidStreamId {
                expected: "0",
                found: stream_id,
            });
        }
        let ttl = eat_u31(buf)?;
        let number_of_requests = eat_u31(buf)?;
        let metadata = match buf.remaining() {
            0 => None,
            len => Some(eat_bytes(buf, len)?),
        };
        Ok(LeaseFrame { ttl, number_of_requests, metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_id() {
        assert_eq!(LeaseFrame::STREAM_ID, 0);
    }

    #[test]
    fn test_codec() {
        let lease = LeaseFrame::new(10, 20, Some(Bytes::from("metadata")));

        let mut buf = BytesMut::new();
        lease.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(ttl): 4
        // len(number_of_requests): 4
        // len(metadata): 8
        let buf_len = buf.len();
        assert_eq!(buf_len, 4 + 2 + 4 + 4 + 8);

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(stream_id, 0);
        assert_eq!(frame_type, FrameType::LEASE);
        assert_eq!(flags, Flags::METADATA);

        let decoded = LeaseFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, lease);
        assert_eq!(lease.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
