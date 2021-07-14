use super::*;
use crate::prelude::DEFAULT_MIMETYPE;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::Duration;
use crate::consts::{DEFAULT_KEEPALIVE_INTERVAL, DEFAULT_KEEPALIVE_TIMEOUT};

/// The setup frame.
///
/// The SETUP frame is sent by the client to inform the server of the parameters under which it
/// desires to operate.
///
/// # Frame Contents
///
/// The lease frame is structured as follows:
///
/// ```text
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Stream ID = 0                         |
/// +-----------+-+-+-+-+-----------+-------------------------------+
/// |Frame Type |0|M|R|L|  Flags    |
/// +-----------+-+-+-+-+-----------+-------------------------------+
/// |         Major Version         |        Minor Version          |
/// +-------------------------------+-------------------------------+
/// |0|                 Time Between KEEPALIVE Frames               |
/// +---------------------------------------------------------------+
/// |0|                       Max Lifetime                          |
/// +---------------------------------------------------------------+
/// |         Token Length          | Resume Identification Token  ...
/// +---------------+-----------------------------------------------+
/// |  MIME Length  |   Metadata Encoding MIME Type                ...
/// +---------------+-----------------------------------------------+
/// |  MIME Length  |     Data Encoding MIME Type                  ...
/// +---------------+-----------------------------------------------+
///                     Metadata & Setup Payload
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupFrame {
    pub(crate) flags: Flags,
    pub(crate) version: Version,
    pub(crate) keepalive_interval: u32,
    pub(crate) keepalive_timeout: u32,
    pub(crate) resume_token: Option<Bytes>,
    pub(crate) metadata_mimetype: Bytes,
    pub(crate) data_mimetype: Bytes,
    pub(crate) payload: Payload,
}

impl SetupFrame {
    /// Setup frames MUST always use Stream ID 0 as they pertain to the connection.
    pub const STREAM_ID: u32 = 0;

    /// Type of this frame.
    pub const TYPE: FrameType = FrameType::SETUP;

    /// Returns a [`SetupFrameBuilder`].
    pub fn builder() -> SetupFrameBuilder {
        SetupFrameBuilder::default()
    }

    /// Returns true if the flags have the LEASE bit set.
    pub fn is_lease(&self) -> bool {
        self.flags.contains(Flags::LEASE)
    }

    /// Returns true if the flags have the RESUME bit set.
    pub fn is_resume(&self) -> bool {
        self.flags.contains(Flags::RESUME)
    }

    /// Returns the protocol version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the time between KEEPALIVE frames that the client will send.
    pub fn keepalive_interval(&self) -> Duration {
        Duration::from_millis(self.keepalive_interval as u64)
    }

    /// Returns the time that a client will allow a server to not respond to
    /// a KEEPALIVE before it is assumed to be dead.
    pub fn keepalive_timeout(&self) -> Duration {
        Duration::from_millis(self.keepalive_timeout as u64)
    }

    /// Returns the resume identification token (not present if the RESUME bit is not set).
    pub fn resume_token(&self) -> Option<&Bytes> {
        self.resume_token.as_ref()
    }

    /// Returns the MIME type for encoding the medadata.
    ///
    /// Note that this will always return `None` if metadata MIME type is not an ASCII string.
    pub fn metadata_mimetype(&self) -> Option<&str> {
        match std::str::from_utf8(&self.metadata_mimetype).ok() {
            Some(x) => {
                if x.is_ascii() {
                    Some(x)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Returns the MIME type for encoding the data.
    ///
    /// Note that this will always return `None` if metadata MIME type is not an ASCII string.
    pub fn data_mimetype(&self) -> Option<&str> {
        match std::str::from_utf8(&self.data_mimetype).ok() {
            Some(x) => {
                if x.is_ascii() {
                    Some(x)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Returns the metadata attached to this frame, if any.
    pub fn metadata(&self) -> Option<&Bytes> {
        self.payload.metadata()
    }

    /// Returns the data attached to this frame, if any.
    pub fn data(&self) -> Option<&Bytes> {
        self.payload.data()
    }

    /// Returns the payload of this setup frame.
    pub fn payload(self) -> Payload {
        self.payload
    }
}

impl Encode for SetupFrame {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(SetupFrame::STREAM_ID);
        buf.put_u16(FrameType::SETUP.bits() | self.flags.bits());
        self.version.encode(buf);
        buf.put_u32(self.keepalive_interval);
        buf.put_u32(self.keepalive_timeout);
        if let Some(token) = &self.resume_token {
            buf.put_u16(token.len() as u16);
            buf.put_slice(token);
        }
        buf.put_u8(self.metadata_mimetype.len() as u8);
        buf.put_slice(&self.metadata_mimetype);
        buf.put_u8(self.data_mimetype.len() as u8);
        buf.put_slice(&self.data_mimetype);
        let u24 = U24::from_usize(
            self.payload.metadata().map(|v| v.len()).unwrap_or_default(),
        );
        buf.put_u8(u24.0);
        buf.put_u16(u24.1);
        self.payload.encode(buf);
    }

    fn len(&self) -> usize {
        // len(stream_id): 4
        // len(frame_type & flags): 2
        // len(version): 4
        // len(keepalive): 4
        // len(lifetime): 4
        // len(token_length): 2
        let mut len = 20;

        // len(resume_token)
        if let Some(resume_token) = &self.resume_token {
            len += resume_token.len();
        }

        // len(mime_metadata_length): 1
        // len(mime_metadata)
        // len(mime_data_length): 1
        // len(mime_data)
        len += 1 + self.metadata_mimetype.len() + 1 + self.data_mimetype.len();

        // len(metadata_length): 3
        // len(payload)
        len += 3 + self.payload.len();

        len
    }
}

impl Decode for SetupFrame {
    type Value = Self;

    fn decode<B: Buf>(
        buf: &mut B,
        _stream_id: u32,
        flags: Flags,
    ) -> Result<Self::Value> {
        let version = eat_version(buf)?;
        let keepalive = eat_u31(buf)?;
        let lifetime = eat_u31(buf)?;
        let resume_token = eat_resume_token(buf, flags)?;
        let metadata_mimetype_len = eat_u8(buf)?;
        let metadata_mimetype =
            eat_bytes(buf, metadata_mimetype_len as usize)?;
        let data_mimetype_len = eat_u8(buf)?;
        let data_mimetype = eat_bytes(buf, data_mimetype_len as usize)?;
        let payload = eat_payload(buf, true)?;
        Ok(SetupFrame {
            flags,
            version,
            keepalive_interval: keepalive,
            keepalive_timeout: lifetime,
            resume_token,
            metadata_mimetype,
            data_mimetype,
            payload,
        })
    }
}

/// A builder for configuring the setup frame.
#[derive(Debug)]
pub struct SetupFrameBuilder {
    flags: Flags,
    version: Version,
    keepalive_interval: u32,
    keepalive_timeout: u32,
    resume_token: Option<Bytes>,
    metadata_mimetype: Bytes,
    data_mimetype: Bytes,
    payload: Payload,
}

impl Default for SetupFrameBuilder {
    fn default() -> SetupFrameBuilder {
        SetupFrameBuilder {
            flags: Flags::empty(),
            version: Version::default(),
            keepalive_interval: DEFAULT_KEEPALIVE_INTERVAL.as_millis() as u32,
            keepalive_timeout: DEFAULT_KEEPALIVE_TIMEOUT.as_millis() as u32,
            resume_token: None,
            metadata_mimetype: Bytes::from(DEFAULT_MIMETYPE),
            data_mimetype: Bytes::from(DEFAULT_MIMETYPE),
            payload: Payload::default(),
        }
    }
}

impl SetupFrameBuilder {
    /// Sets the `Resume` flag.
    pub fn set_resume_flag(mut self) -> Self {
        self.flags |= Flags::RESUME;
        self
    }

    /// Sets the `Lease` flag.
    pub fn set_lease_flag(mut self) -> Self {
        self.flags |= Flags::LEASE;
        self
    }

    /// Sets the RSocket protocol version.
    pub fn set_version(mut self, major: u16, minor: u16) -> Self {
        self.version = Version::new(major, minor);
        self
    }

    /// Sets the interval (in milliseconds) between two KEEPALIVE frames that the client will send.
    /// This value MUST be > `0` and <= [`MAX_U31`].
    ///
    /// - For server-to-server connections, a reasonable time interval between client KEEPALIVE
    /// frames is 500ms.
    ///
    /// - For mobile-to-server connections, the time interval between client KEEPALIVE frames is
    /// often > 30,000ms.
    pub fn set_keepalive_interval(mut self, interval: u32) -> Self {
        debug_assert_max_u31!(interval);
        self.keepalive_interval = interval & MAX_U31;
        self
    }

    /// Sets the time (in milliseconds) that a client will allow a server to not respond to a
    /// KEEPALIVE before it is assumed to be dead. This value MUST be > 0 and <= [`MAX_U31`].
    pub fn set_keepalive_timeout(mut self, timeout: u32) -> Self {
        debug_assert_max_u31!(timeout);
        self.keepalive_timeout = timeout & MAX_U31;
        self
    }

    /// Sets the resume identification token.
    ///
    /// # Panics
    ///
    /// This function panics if the length of the given token is greater than 65,535 bytes.
    pub fn set_resume_token(mut self, token: Bytes) -> Self {
        assert!(token.len() <= 65_535);
        self.resume_token = Some(token);
        self.flags |= Flags::RESUME;
        self
    }

    /// Sets the metadata mimetype.
    ///
    /// The given mimetype should be a ASCII string that includes the [`Internet media type`]
    /// specified in [`RFC 2045`].
    ///
    /// # Panics
    ///
    /// This function panics if the length of the given `mimetype` is greater than `256` bytes.
    ///
    /// [`Internet media type`]: https://en.wikipedia.org/wiki/Internet_media_type
    /// [`RFC 2045`]: https://datatracker.ietf.org/doc/html/rfc2045
    pub fn set_metadata_mimetype<T>(mut self, mimetype: T) -> Self
    where
        T: Into<String>,
    {
        let mimetype: String = mimetype.into();
        assert!(mimetype.len() <= 256);
        self.metadata_mimetype = Bytes::from(mimetype);
        self
    }

    /// Sets the data mimetype.
    ///
    /// The given mimetype should be a ASCII string that includes the [`Internet media type`]
    /// specified in [`RFC 2045`].
    ///
    /// # Panics
    ///
    /// This function panics if the length of the given `mimetype` is greater than `256` bytes.
    ///
    /// [`Internet media type`]: https://en.wikipedia.org/wiki/Internet_media_type
    /// [`RFC 2045`]: https://datatracker.ietf.org/doc/html/rfc2045
    pub fn set_data_mimetype<T>(mut self, mimetype: T) -> Self
    where
        T: Into<String>,
    {
        let mimetype: String = mimetype.into();
        assert!(mimetype.len() <= 256);
        self.data_mimetype = Bytes::from(mimetype);
        self
    }

    /// Sets the metadata payload of this setup frame.
    pub fn set_metadata(mut self, metadata: Bytes) -> Self {
        self.flags |= Flags::METADATA;
        self.payload.metadata = Some(metadata);
        self
    }

    /// Sets the data payload of this setup frame.
    pub fn set_data(mut self, data: Bytes) -> Self {
        self.payload.data = Some(data);
        self
    }

    /// Builds a [`SetupFrame`] from this builder.
    pub fn build(self) -> SetupFrame {
        SetupFrame {
            flags: self.flags,
            version: self.version,
            keepalive_interval: self.keepalive_interval,
            keepalive_timeout: self.keepalive_timeout,
            resume_token: self.resume_token,
            metadata_mimetype: self.metadata_mimetype,
            data_mimetype: self.data_mimetype,
            payload: self.payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_id() {
        assert_eq!(SetupFrame::STREAM_ID, 0);
    }

    #[test]
    fn test_codec() {
        let setup = SetupFrame::builder()
            .set_version(1, 0)
            .set_keepalive_interval(1000)
            .set_keepalive_timeout(2000)
            .set_resume_flag()
            .set_lease_flag()
            .set_resume_token(Bytes::from("resume token".to_string()))
            .set_metadata_mimetype("application/json")
            .set_data_mimetype("application/binary")
            .set_metadata(Bytes::from("metadata"))
            .set_data(Bytes::from("data"))
            .build();

        let mut buf = BytesMut::new();
        setup.encode(&mut buf);
        let mut buf = buf.freeze();

        // len(stream_id): 4
        // len(flags): 2
        // len(version): 4
        // len(keepalive): 4
        // len(lifetime): 4
        // len(resume_token_lenght): 2
        // len(resume_token): 12
        // len(metadata_mimetype_len): 1
        // len(metadata_mimetype): 16
        // len(data_mimetype_len): 1
        // len(data_mimetype): 18
        // len(metadata_len): 3
        // len(metadata): 8
        // len(data): 4
        let buf_len = buf.len();
        assert_eq!(
            buf_len,
            4 + 2 + 4 + 4 + 4 + 2 + 12 + 1 + 16 + 1 + 18 + 3 + 8 + 4
        );

        // Eat the stream_id and flags before decoding bytes.
        let stream_id = eat_stream_id(&mut buf).unwrap();
        let (frame_type, flags) = eat_flags(&mut buf).unwrap();
        assert_eq!(stream_id, 0);
        assert_eq!(frame_type, FrameType::SETUP);
        assert_eq!(flags, Flags::METADATA | Flags::RESUME | Flags::LEASE);

        let decoded = SetupFrame::decode(&mut buf, stream_id, flags).unwrap();

        assert_eq!(decoded, setup);
        assert_eq!(setup.len(), buf_len);
        assert_eq!(decoded.len(), buf_len);
    }
}
