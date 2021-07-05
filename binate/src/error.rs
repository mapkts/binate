//! RSocket error and result types.
use crate::frame::DecodeError;
use std::error::Error as StdError;
use std::fmt;
use std::io;

/// A Result type aliased for [`Result`]<T, [`Error`]>.
///
/// [`Result`]: std::result::Result
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when handling RSocket streams.
pub struct Error {
    inner: Box<ErrorImpl>,
}

type Source = Box<dyn Send + Sync + StdError>;

struct ErrorImpl {
    kind: Kind,
    source: Option<Source>,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub(crate) enum Kind {
    // Decode errors
    Decode(DecodeError),

    // Protocol errors
    InvalidSetup,
    UnsupportedSetup,
    RejectedSetup,
    RejectedResume,
    ConnectionError,
    ConnectionClose,
    ApplicationError,
    Rejected,
    Canceled,
    Invalid,

    // IO errors
    Io,
}

/// A list of valid RSocket protocol error codes.
///
/// See [`here`] for more information about RSocket error codes.
///
/// [`here`]: https://github.com/rsocket/rsocket/blob/master/Protocol.md#error-codes
#[non_exhaustive]
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    /// The Setup frame is invalid for the server
    /// (it could be that the client is too recent for the old server).
    InvalidSetup       = 0x00000001,
    /// Some (or all) of the parameters specified by the client are unsupported by the server.
    UnsupportedSetup   = 0x00000002,
    /// The server rejected the setup, it can specify the reason in the payload. 
    RejectedSetup      = 0x00000003,
    /// The server rejected the resume, it can specify the reason in the payload.
    RejectedResume     = 0x00000004,
    /// The connection is being terminated. Sender or Receiver of this frame MAY close the 
    /// connection immediately without waiting for outstanding streams to terminate.
    ConnectionError    = 0x00000101,
    /// The connection is being terminated. Sender or Receiver of this frame MUST wait for
    /// outstanding streams to terminate before closing the connection. New requests MAY not be 
    /// accepted.
    ConnectionClose    = 0x00000102,
    /// Application layer logic generating a Reactive Streams onError event.
    ApplicationError   = 0x00000201,
    /// Despite being a valid request, the Responder decided to reject it. 
    /// The Responder guarantees that it didn't process the request.
    Rejected           = 0x00000202,
    /// The Responder canceled the request but may have started processing it 
    /// (similar to REJECTED but doesn't guarantee lack of side-effects).
    Canceled           = 0x00000203,
    /// The request is invalid.
    Invalid            = 0x00000204,
}

impl Error {
    pub(crate) fn new<E>(kind: Kind, source: Option<E>) -> Error
    where
        E: Into<Source>,
    {
        Error {
            inner: Box::new(ErrorImpl {
                kind,
                source: source.map(Into::into),
            }),
        }
    }

    /// Returns true if this error is related to decoding `Bytes`.
    pub fn is_decode(&self) -> bool {
        matches!(self.inner.kind, Kind::Decode(_))
    }

    /// Returns true if this error is a RSocket protocol error.
    pub fn is_protocol(&self) -> bool {
        use Kind::*;
        matches!(
            self.inner.kind,
            InvalidSetup
                | UnsupportedSetup
                | RejectedSetup
                | RejectedResume
                | ConnectionError
                | ConnectionClose
                | ApplicationError
                | Rejected
                | Canceled
                | Invalid
        )
    }

    /// Returns true if this error is related to connection setup.
    pub fn is_setup(&self) -> bool {
        matches!(
            self.inner.kind,
            Kind::InvalidSetup | Kind::UnsupportedSetup | Kind::RejectedSetup
        )
    }

    /// Returns true if this is protocol error `INVALID_SETUP`.
    pub fn is_invalid_setup(&self) -> bool {
        matches!(self.inner.kind, Kind::InvalidSetup)
    }

    /// Returns true if this is protocol error `UNSUPPORTED_SETUP`.
    pub fn is_unsupported_setup(&self) -> bool {
        matches!(self.inner.kind, Kind::UnsupportedSetup)
    }

    /// Returns true if this is protocol error `REJECTED_SETUP`.
    pub fn is_rejected_setup(&self) -> bool {
        matches!(self.inner.kind, Kind::RejectedSetup)
    }

    /// Returns true if this is protocol error `REJECTED_RESUME`.
    pub fn is_rejected_resume(&self) -> bool {
        matches!(self.inner.kind, Kind::RejectedResume)
    }

    /// Returns true if this is protocol error `CONNECTION_ERROR`.
    ///
    /// Sender or Receiver of this error MAY close the connection immediately without waiting
    /// for outstanding streams to terminate.
    pub fn is_connection_error(&self) -> bool {
        matches!(self.inner.kind, Kind::ConnectionError)
    }

    /// Returns true if this is protocol error `CONNECTION_CLOSE`.
    ///
    /// Sender or Receiver of this error MUST wait for outstanding streams to terminate before
    /// closing the connection. New requests MAY not be accepted.
    pub fn is_connection_close(&self) -> bool {
        matches!(self.inner.kind, Kind::ConnectionClose)
    }

    /// Returns true if this is protocol error `APPLICATION_ERROR`.
    pub fn is_application_error(&self) -> bool {
        matches!(self.inner.kind, Kind::ApplicationError)
    }

    /// Returns true if this is protocol error `REJECTED`.
    pub fn is_rejected(&self) -> bool {
        matches!(self.inner.kind, Kind::Rejected)
    }

    /// Returns true if this is protocol error `CANCELED`.
    pub fn is_cancel(&self) -> bool {
        matches!(self.inner.kind, Kind::Canceled)
    }

    /// Returns true if this is protocol error `INVALID`.
    pub fn is_invalid(&self) -> bool {
        matches!(self.inner.kind, Kind::Invalid)
    }

    fn description(&self) -> &str {
        use Kind::*;
        match &self.inner.kind {
            InvalidSetup => "INVALID_SETUP (0x00000001)",
            UnsupportedSetup => "UNSUPPORTED_SETUP (0x00000002)",
            RejectedSetup => "REJECTED_SETUP (0x00000003)",
            RejectedResume => "REJECTED_RESUME (0x00000004)",
            ConnectionError => "CONNECTION_ERROR (0x00000101)",
            ConnectionClose => "CONNECTION_CLOSE (0x00000102)",
            ApplicationError => "APPLICATION_ERROR (0x00000201)",
            Rejected => "CONNECTION_ERROR (0x00000202)",
            Canceled => "CANCELED (0x00000203)",
            Invalid => "INVALID (0x00000204)",
            Decode(_) => "error decoding frame",
            Io => "I/O error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref source) = self.inner.source {
            write!(f, "{}: {}", self.description(), source)
        } else {
            f.write_str(self.description())
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_tuple("nightwatch_rsocket::Error");
        f.field(&self.inner.kind);
        if let Some(ref source) = self.inner.source {
            f.field(source);
        }
        f.finish()
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Error {
        let source = e.to_string();
        Error::new(Kind::Decode(e), Some(source))
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::new(Kind::Io, Some(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
    }

    #[test]
    fn assert_send_sync() {
        assert_send::<Error>();
        assert_sync::<Error>();
    }

    #[test]
    fn from_decode_error() {
        let decode = DecodeError::InComplete;
        let actual: Error = decode.clone().into();
        match actual.inner.kind {
            Kind::Decode(e) => assert_eq!(e, decode),
            _ => panic!("{:?}", actual),
        }
        assert!(actual.inner.source.is_some());
    }
}
