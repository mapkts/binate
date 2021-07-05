//! Provides the [`Frame`] type that represents a RSocket protocol frame, and utilities
//! for encoding/decoding frames into/from byte arrays.
pub mod codec;

mod decode;
mod encode;
mod flags;
mod payload;
mod u24;
mod version;
mod visit;

pub use self::decode::{Decode, DecodeError};
pub use self::encode::Encode;
pub use self::flags::{Flags, FrameType};
pub use self::payload::{
    Data, Metadata, Payload, PayloadBuilder, PayloadChunks,
};
pub use self::u24::U24;
pub use self::version::Version;

use bytes::{Buf, BytesMut};
use codec::*;
use visit::*;

/// The maximum value 31-bit unsigned integer can hold.
pub const MAX_U31: u32 = 0x7FFFFFFF;

/// The maximum value 63-bit unsigned integer can hold.
pub const MAX_U63: u64 = 0x7FFF_FFFF_FFFF_FFFF;

/// A frame in the RSocket protocol.
///
/// A frame is a single message in the RSocket protocol, which can be a request, response or
/// protocol processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// The SETUP frame.
    Setup(SetupFrame),
    /// The ERROR frame.
    Error(ErrorFrame),
    /// The LEASE frame.
    Lease(LeaseFrame),
    /// The KEEPALIVE frame.
    Keepalive(KeepaliveFrame),
    /// The REQUEST_RESPONSE frame.
    RequestResponse(RequestResponseFrame),
    /// The REQUEST_FNF frame.
    RequestFnf(RequestFnfFrame),
    /// The REQUEST_STREAM frame.
    RequestStream(RequestStreamFrame),
    /// The REQUEST_CHANNEL frame.
    RequestChannel(RequestChannelFrame),
    /// The REQUEST_N frame.
    RequestN(RequestNFrame),
    /// The CANCEL frame.
    Cancel(CancelFrame),
    /// The PAYLOAD frame.
    Payload(PayloadFrame),
    /// The METADATA_PUSH frame.
    MetadataPush(MetadataPushFrame),
    /// The RESUME frame.
    Resume(ResumeFrame),
    /// The RESUME_OK frame.
    ResumeOk(ResumeOkFrame),
}

impl Encode for Frame {
    fn encode(&self, buf: &mut BytesMut) {
        match self {
            Frame::Setup(v) => v.encode(buf),
            Frame::Error(v) => v.encode(buf),
            Frame::Lease(v) => v.encode(buf),
            Frame::Keepalive(v) => v.encode(buf),
            Frame::RequestResponse(v) => v.encode(buf),
            Frame::RequestFnf(v) => v.encode(buf),
            Frame::RequestStream(v) => v.encode(buf),
            Frame::RequestChannel(v) => v.encode(buf),
            Frame::RequestN(v) => v.encode(buf),
            Frame::Cancel(v) => v.encode(buf),
            Frame::Payload(v) => v.encode(buf),
            Frame::MetadataPush(v) => v.encode(buf),
            Frame::Resume(v) => v.encode(buf),
            Frame::ResumeOk(v) => v.encode(buf),
        }
    }

    fn len(&self) -> usize {
        match self {
            Frame::Setup(v) => v.len(),
            Frame::Error(v) => v.len(),
            Frame::Lease(v) => v.len(),
            Frame::Keepalive(v) => v.len(),
            Frame::RequestResponse(v) => v.len(),
            Frame::RequestFnf(v) => v.len(),
            Frame::RequestStream(v) => v.len(),
            Frame::RequestChannel(v) => v.len(),
            Frame::RequestN(v) => v.len(),
            Frame::Cancel(v) => v.len(),
            Frame::Payload(v) => v.len(),
            Frame::MetadataPush(v) => v.len(),
            Frame::Resume(v) => v.len(),
            Frame::ResumeOk(v) => v.len(),
        }
    }
}

impl Frame {
    /// Decode the given bytes into a frame.
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        if buf.remaining() < 6 {
            return Err(DecodeError::InComplete);
        }
        let stream_id = eat_stream_id(buf)?;
        let (frame_type, flags) = eat_flags(buf)?;
        match frame_type {
            FrameType::SETUP
            | FrameType::LEASE
            | FrameType::KEEPALIVE
            | FrameType::METADATA_PUSH
            | FrameType::RESUME
            | FrameType::RESUME_OK => {
                if stream_id != 0 {
                    return Err(DecodeError::InvalidStreamId {
                        expected: "0",
                        found: stream_id,
                    });
                }
            }
            _ => (),
        }

        Ok(match frame_type {
            FrameType::SETUP => {
                Frame::Setup(SetupFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::ERROR => {
                Frame::Error(ErrorFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::LEASE => {
                Frame::Lease(LeaseFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::KEEPALIVE => Frame::Keepalive(KeepaliveFrame::decode(
                buf, stream_id, flags,
            )?),
            FrameType::REQUEST_RESPONSE => Frame::RequestResponse(
                RequestResponseFrame::decode(buf, stream_id, flags)?,
            ),
            FrameType::REQUEST_FNF => Frame::RequestFnf(
                RequestFnfFrame::decode(buf, stream_id, flags)?,
            ),
            FrameType::REQUEST_STREAM => Frame::RequestStream(
                RequestStreamFrame::decode(buf, stream_id, flags)?,
            ),
            FrameType::REQUEST_CHANNEL => Frame::RequestChannel(
                RequestChannelFrame::decode(buf, stream_id, flags)?,
            ),
            FrameType::REQUEST_N => {
                Frame::RequestN(RequestNFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::CANCEL => {
                Frame::Cancel(CancelFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::PAYLOAD => {
                Frame::Payload(PayloadFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::METADATA_PUSH => Frame::MetadataPush(
                MetadataPushFrame::decode(buf, stream_id, flags)?,
            ),
            FrameType::RESUME => {
                Frame::Resume(ResumeFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::RESUME_OK => {
                Frame::ResumeOk(ResumeOkFrame::decode(buf, stream_id, flags)?)
            }
            FrameType::EXT => {
                unimplemented!()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_max_u31() {
        assert_eq!(MAX_U31, u32::MAX >> 1);
    }

    #[test]
    fn test_max_63() {
        assert_eq!(MAX_U63, u64::MAX >> 1);
    }

    #[test]
    fn test_frame_decode() {
        let f = RequestFnfFrame::new(
            1,
            true,
            Payload::builder()
                .set_metadata(Bytes::from("metadata"))
                .set_data(Bytes::from("data"))
                .build(),
        );

        let mut buf = BytesMut::new();
        f.encode(&mut buf);
        let mut buf = buf.freeze();

        let decoded = Frame::decode(&mut buf).unwrap();
        assert_eq!(decoded, Frame::RequestFnf(f));
    }
}
