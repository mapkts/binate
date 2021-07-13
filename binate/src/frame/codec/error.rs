use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// The error frame.
///
/// Error frames are used for errors on individual requests/streams as well as connection errors
/// and in response to SETUP frames.
///
/// # Frame Contents
///
/// The error frame is structured as follows:
///
/// ```text
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Stream ID                           |
/// +-----------+-+-+---------------+-------------------------------+
/// |Frame Type |0|0|      Flags    |
/// +-----------+-+-+---------------+-------------------------------+
/// |                          Error Code                           |
/// +---------------------------------------------------------------+
///                            Error Data
/// ```
///
/// A Stream ID of 0 means the error pertains to the connection., including connection
/// establishment. A Stream ID > 0 means the error pertains to a given stream.
///
/// The Error Data is typically an Exception message, but could include stringified stacktrace
/// information if appropriate.
///
/// See the [`Frame Error`] section of the RSocket protocol spec for more information.
///
/// [`Frame Error`]: https://rsocket.io/about/protocol/#error-frame-0x0b
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorFrame {
    stream_id: u32,
    code: u32,
    data: Option<Bytes>,
}

impl ErrorFrame {
    /// The Setup frame is invalid for the server (it could be that the client is too recent for
    /// the old server).
    ///
    /// Stream ID MUST be 0.
    pub const INVALID_SETUP: u32 = 0x00000001;

    /// Some (or all) of the parameters specified by the client are unsupported by the server.
    ///
    /// Stream ID MUST be 0.
    pub const UNSUPPORTED_SETUP: u32 = 0x00000002;

    /// The server rejected the setup, it can specify the reason in the payload.
    ///
    /// Stream ID MUST be 0.
    pub const REJECTED_SETUP: u32 = 0x00000003;

    /// The server rejected the resume, it can specify the reason in the payload.
    ///
    /// Stream ID MUST be 0.
    pub const REJECTED_RESUME: u32 = 0x00000004;

    /// The connection is being terminated.
    ///
    /// Sender or Receiver of this frame MAY close the connection immediately without waiting
    /// for outstanding streams to terminate. Stream ID MUST be 0.
    pub const CONNECTION_ERROR: u32 = 0x00000101;

    /// The connection is being terminated.
    ///
    /// Sender or Receiver of this frame MUST wait for outstanding streams to terminate before
    /// closing the connection. New requests MAY not be accepted. Stream ID MUST be 0.
    pub const CONNECTION_CLOSE: u32 = 0x00000102;

    /// Application layer logic generating a [`Reactive Streams`] onError event.
    ///
    /// Stream ID MUST be > 0.
    ///
    /// [`Reactive Streams`]: http://www.reactive-streams.org/
    pub const APPLICATION_ERROR: u32 = 0x00000201;

    /// Despite being a valid request, the Responder decided to reject it.
    ///
    /// The Responder guarantees that it didn't process the request. The reason for the rejection
    /// is explained in the Error Data section. Stream ID MUST be > 0.
    pub const REJECTED: u32 = 0x00000202;

    /// The Responder canceled the request but may have started processing it
    /// (similar to REJECTED but doesn't guarantee lack of side-effects).
    ///
    /// Stream ID MUST be > 0.
    pub const CANCELED: u32 = 0x00000203;

    /// The request is invalid. Stream ID MUST be > 0.
    pub const INVALID: u32 = 0x00000204;

    /// The minimum error code that can used as application layer error.
    pub const MIN_APPLICATION_ERROR_CODE: u32 = 0x00000301;

    /// The maximum error code that can used as application layer error.
    pub const MAX_APPLICATION_ERROR_CODE: u32 = 0xFFFFFFFE;

    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::ERROR;
}

impl ErrorFrame {
    /// Create a new `ErrorFrame`.
    ///
    /// - `stream_id` and `error_code` MUST be <= [`MAX_U31`].
    /// - `data` SHOULD be a UTF-8 encoded string.
    pub fn new(stream_id: u32, error_code: u32, data: Option<Bytes>) -> Self {
        debug_assert_max_u31!(stream_id);
        debug_assert_max_u31!(error_code);
        ErrorFrame {
            stream_id: stream_id & MAX_U31,
            code: error_code & MAX_U31,
            data,
        }
    }

    /// Returns the stream ID of this frame.
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }

    /// Returns the error code in this error frame.
    pub fn error_code(&self) -> u32 {
        self.code
    }

    /// Returns the error data attached to this frame, if any.
    pub fn data(&self) -> Option<&Bytes> {
        self.data.as_ref()
    }

    /// Returns the error data in this error frame in UTF-8 format. If the error data is not valid
    /// UTF-8, this will return `None`.
    pub fn data_utf8(&self) -> Option<&str> {
        self.data.as_ref().map(|data| std::str::from_utf8(data).ok()).flatten()
    }
}

impl Encode for ErrorFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(self.stream_id);
        buf.put_u16(FrameType::ERROR.bits());
        buf.put_u32(self.code);
        if let Some(bytes) = &self.data {
            buf.put_slice(bytes);
        }
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(frame_type & flags): 2
        // len(error_code): 4
        let mut len = 10;

        // len(error_data)
        if let Some(bytes) = &self.data {
            len += bytes.len()
        }

        len
    }
}

impl Decode for ErrorFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        stream_id: u32,
        _flags: Flags,
    ) -> Result<Self::Value> {
        let code = eat_u32(buf)?;
        validate_stream_id(stream_id, code)?;
        let data = match buf.remaining() {
            0 => None,
            len => Some(eat_bytes(buf, len)?),
        };
        Ok(ErrorFrame { stream_id, code, data })
    }
}

fn validate_stream_id(stream_id: u32, code: u32) -> Result<()> {
    match code {
        ErrorFrame::INVALID_SETUP
        | ErrorFrame::UNSUPPORTED_SETUP
        | ErrorFrame::REJECTED_SETUP
        | ErrorFrame::REJECTED_RESUME
        | ErrorFrame::CONNECTION_ERROR
        | ErrorFrame::CONNECTION_CLOSE => {
            if stream_id != 0 {
                return Err(DecodeError::InvalidStreamId {
                    expected: "0",
                    found: stream_id,
                });
            }
        }
        ErrorFrame::APPLICATION_ERROR
        | ErrorFrame::REJECTED
        | ErrorFrame::CANCELED
        | ErrorFrame::INVALID => {
            if stream_id == 0 {
                return Err(DecodeError::InvalidStreamId {
                    expected: "> 0",
                    found: stream_id,
                });
            }
        }
        _ => (),
    }

    Ok(())
}
