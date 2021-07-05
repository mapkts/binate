use super::Flags;
use bytes::Buf;
use std::error::Error as StdError;
use std::fmt;

/// A trait for decoding bytes into a frame.
pub trait Decode {
    /// The value decoded into.
    type Value;

    /// Decodes the given bytes into a frame.
    fn decode<B: Buf>(
        bytes: &mut B,
        stream_id: u32,
        flags: Flags,
    ) -> Result<Self::Value, DecodeError>;
}

/// Errors that can occur when decoding bytes into a specific frame failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    /// Not enough data is available to parse a frame.
    InComplete,
    /// The decoded frame type is unrecognized.
    UnrecognizedFrameType(u16),
    /// The decoded stream ID is invalid.
    InvalidStreamId {
        /// expected stream ID
        expected: &'static str,
        /// found stream ID
        found: u32,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DecodeError::*;
        match self {
            InComplete => write!(f, "incomplete frame"),
            UnrecognizedFrameType(v) => {
                write!(f, "unrecognized frame type {0:#x}", v)
            }
            InvalidStreamId { expected, found } => write!(
                f,
                "invalid stream ID (expected {}, found {})",
                expected, found
            ),
        }
    }
}

impl StdError for DecodeError {}
