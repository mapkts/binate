/// Type of frame.
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// Sent by client to initiate protocol processing.
    SETUP,
    /// Sent by Responder to grant the ability to send requests.
    LEASE,
    /// Connection keepalive.
    KEEPALIVE,
    /// Request single response.
    REQUEST_RESPONSE,
    /// A single one-way message.
    REQUEST_FNF,
    /// Request a completable stream.
    REQUEST_STREAM,
    /// Request a completable stream in both directions.
    REQUEST_CHANNEL,
    /// Request N more items with Reactive Streams semantics.
    REQUEST_N,
    /// Cancel outstanding request.
    CANCEL,
    /// Payload on a stream. For example, response to a request, or message on a channel.
    PAYLOAD,
    /// Error at connection or application level.
    ERROR,
    /// Asynchronous Metadata frame
    METADATA_PUSH,
    /// Replaces SETUP for Resuming Operation (optional)
    RESUME,
    /// Sent in response to a RESUME if resuming operation possible (optional)
    RESUME_OK,
    /// Used To Extend more frame types as well as extensions.
    EXT,
}

impl FrameType {
    /// Specify the frame type with the raw value described in the [`Frame Types`] section of the
    /// RSocket protocol spec. This will return `None` if the given raw value is unrecognized.
    ///
    /// [`Frame Types`]: https://rsocket.io/about/protocol/#frame-types
    pub fn from_value(val: u16) -> Option<FrameType> {
        match val {
            0x01 => Some(FrameType::SETUP),
            0x02 => Some(FrameType::LEASE),
            0x03 => Some(FrameType::KEEPALIVE),
            0x04 => Some(FrameType::REQUEST_RESPONSE),
            0x05 => Some(FrameType::REQUEST_FNF),
            0x06 => Some(FrameType::REQUEST_STREAM),
            0x07 => Some(FrameType::REQUEST_CHANNEL),
            0x08 => Some(FrameType::REQUEST_N),
            0x09 => Some(FrameType::CANCEL),
            0x0A => Some(FrameType::PAYLOAD),
            0x0B => Some(FrameType::ERROR),
            0x0C => Some(FrameType::METADATA_PUSH),
            0x0D => Some(FrameType::RESUME),
            0x0E => Some(FrameType::RESUME_OK),
            0x3F => Some(FrameType::EXT),
            _ => None,
        }
    }

    /// Convert from underlying bit representation, if the bit representation contains a valid
    /// frame type.
    ///
    /// In other words, the higher 6-bit of the given bit pattern will convert to a [`FrameType`].
    ///
    /// ```text
    /// 0                   1                   2                   3
    /// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    /// +-----------+-+-+-+-+-----------+-------------------------------+
    /// |Frame Type |I|M|     Flags     |
    /// +-----------+-+-+-+-+-----------+-------------------------------+
    /// ```
    pub fn from_bits(bits: u16) -> Option<FrameType> {
        match bits >> 10 {
            0x01 => Some(FrameType::SETUP),
            0x02 => Some(FrameType::LEASE),
            0x03 => Some(FrameType::KEEPALIVE),
            0x04 => Some(FrameType::REQUEST_RESPONSE),
            0x05 => Some(FrameType::REQUEST_FNF),
            0x06 => Some(FrameType::REQUEST_STREAM),
            0x07 => Some(FrameType::REQUEST_CHANNEL),
            0x08 => Some(FrameType::REQUEST_N),
            0x09 => Some(FrameType::CANCEL),
            0x0A => Some(FrameType::PAYLOAD),
            0x0B => Some(FrameType::ERROR),
            0x0C => Some(FrameType::METADATA_PUSH),
            0x0D => Some(FrameType::RESUME),
            0x0E => Some(FrameType::RESUME_OK),
            0x3F => Some(FrameType::EXT),
            _ => None,
        }
    }

    /// Returns the corresponding raw value of this frame type. The raw value is described in the
    /// [`Frame Types`] section of the RSocket protocol spec.
    ///
    /// [`Frame Types`]: https://rsocket.io/about/protocol/#frame-types
    pub fn value(self) -> u16 {
        match self {
            FrameType::SETUP => 0x01,
            FrameType::LEASE => 0x02,
            FrameType::KEEPALIVE => 0x03,
            FrameType::REQUEST_RESPONSE => 0x04,
            FrameType::REQUEST_FNF => 0x05,
            FrameType::REQUEST_STREAM => 0x06,
            FrameType::REQUEST_CHANNEL => 0x07,
            FrameType::REQUEST_N => 0x08,
            FrameType::CANCEL => 0x09,
            FrameType::PAYLOAD => 0x0A,
            FrameType::ERROR => 0x0B,
            FrameType::METADATA_PUSH => 0x0C,
            FrameType::RESUME => 0x0D,
            FrameType::RESUME_OK => 0x0E,
            FrameType::EXT => 0x3F,
        }
    }

    /// Convert this frame type to a bit representation, with the higher 6-bit of the return value
    /// setting to the raw value of the frame type.
    ///
    /// ```text
    /// 0                   1                   2                   3
    /// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    /// +-----------+-+-+-+-+-----------+-------------------------------+
    /// |Frame Type |I|M|     Flags     |
    /// +-----------+-+-+-+-+-----------+-------------------------------+
    /// ```
    pub fn bits(self) -> u16 {
        let val = match self {
            FrameType::SETUP => 0x01,
            FrameType::LEASE => 0x02,
            FrameType::KEEPALIVE => 0x03,
            FrameType::REQUEST_RESPONSE => 0x04,
            FrameType::REQUEST_FNF => 0x05,
            FrameType::REQUEST_STREAM => 0x06,
            FrameType::REQUEST_CHANNEL => 0x07,
            FrameType::REQUEST_N => 0x08,
            FrameType::CANCEL => 0x09,
            FrameType::PAYLOAD => 0x0A,
            FrameType::ERROR => 0x0B,
            FrameType::METADATA_PUSH => 0x0C,
            FrameType::RESUME => 0x0D,
            FrameType::RESUME_OK => 0x0E,
            FrameType::EXT => 0x3F,
        };
        val << 10
    }
}

bitflags! {
    /// Frame header flags.
    pub struct Flags: u16 {
        /// The frame can be ignored.
        const IGNORE           = 0b01_0000_0000;
        /// Metadata present.
        const METADATA         = 0b00_1000_0000;
        /// More fragments follow this fragment.
        const FOLLOWS          = 0b00_0100_0000;
        /// Client requests resume capability if possible.
        const RESUME           = 0b00_0100_0000;
        /// Respond with KEEPALIVE.
        const RESPOND          = 0b00_0100_0000;
        /// Bit to indicate stream completion.
        const COMPLETE         = 0b00_0010_0000;
        /// Will honor LEASE.
        const LEASE            = 0b00_0010_0000;
        /// Bit to indicate Next (Payload Data and/or Metadata present).
        const NEXT             = 0b00_0001_0000;
    }
}

impl Flags {
    /// Returns true if the Flags have the IGNORE bit set.
    pub fn is_ignore(&self) -> bool {
        self.contains(Flags::IGNORE)
    }

    /// Returns true if the Flags have the METADATA bit set.
    pub fn is_metadata(&self) -> bool {
        self.contains(Flags::METADATA)
    }

    /// Returns true if the Flags have the FOLLOWS bit set.
    pub fn is_follows(&self) -> bool {
        self.contains(Flags::FOLLOWS)
    }

    /// Returns true if the Flags have the RESUME bit set.
    pub fn is_resume(&self) -> bool {
        self.contains(Flags::RESUME)
    }

    /// Returns true if the Flags have the RESPOND bit set.
    pub fn is_respond(&self) -> bool {
        self.contains(Flags::RESPOND)
    }

    /// Returns true if the Flags have the COMPLETE bit set.
    pub fn is_complete(&self) -> bool {
        self.contains(Flags::COMPLETE)
    }

    /// Returns true if the Flags have the LEASE bit set.
    pub fn is_lease(&self) -> bool {
        self.contains(Flags::LEASE)
    }

    /// Returns true if the Flags have the NEXT bit set.
    pub fn is_next(&self) -> bool {
        self.contains(Flags::NEXT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value() {
        assert_eq!(FrameType::from_value(0x01).unwrap(), FrameType::SETUP);
        assert_eq!(FrameType::from_value(0x02).unwrap(), FrameType::LEASE);
        assert_eq!(FrameType::from_value(0x03).unwrap(), FrameType::KEEPALIVE);
        assert_eq!(
            FrameType::from_value(0x04).unwrap(),
            FrameType::REQUEST_RESPONSE
        );
        assert_eq!(
            FrameType::from_value(0x05).unwrap(),
            FrameType::REQUEST_FNF
        );
        assert_eq!(
            FrameType::from_value(0x06).unwrap(),
            FrameType::REQUEST_STREAM
        );
        assert_eq!(
            FrameType::from_value(0x07).unwrap(),
            FrameType::REQUEST_CHANNEL
        );
        assert_eq!(FrameType::from_value(0x08).unwrap(), FrameType::REQUEST_N);
        assert_eq!(FrameType::from_value(0x09).unwrap(), FrameType::CANCEL);
        assert_eq!(FrameType::from_value(0x0A).unwrap(), FrameType::PAYLOAD);
        assert_eq!(FrameType::from_value(0x0B).unwrap(), FrameType::ERROR);
        assert_eq!(
            FrameType::from_value(0x0C).unwrap(),
            FrameType::METADATA_PUSH
        );
        assert_eq!(FrameType::from_value(0x0D).unwrap(), FrameType::RESUME);
        assert_eq!(FrameType::from_value(0x0E).unwrap(), FrameType::RESUME_OK);
        assert_eq!(FrameType::from_value(0x3F).unwrap(), FrameType::EXT);
        assert!(FrameType::from_value(0x00).is_none());
    }

    #[test]
    fn from_bits() {
        assert_eq!(
            FrameType::from_bits(0x01 << 10).unwrap(),
            FrameType::SETUP
        );
        assert_eq!(
            FrameType::from_bits(0x02 << 10).unwrap(),
            FrameType::LEASE
        );
        assert_eq!(
            FrameType::from_bits(0x03 << 10).unwrap(),
            FrameType::KEEPALIVE
        );
        assert_eq!(
            FrameType::from_bits(0x04 << 10).unwrap(),
            FrameType::REQUEST_RESPONSE
        );
        assert_eq!(
            FrameType::from_bits(0x05 << 10).unwrap(),
            FrameType::REQUEST_FNF
        );
        assert_eq!(
            FrameType::from_bits(0x06 << 10).unwrap(),
            FrameType::REQUEST_STREAM
        );
        assert_eq!(
            FrameType::from_bits(0x07 << 10).unwrap(),
            FrameType::REQUEST_CHANNEL
        );
        assert_eq!(
            FrameType::from_bits(0x08 << 10).unwrap(),
            FrameType::REQUEST_N
        );
        assert_eq!(
            FrameType::from_bits(0x09 << 10).unwrap(),
            FrameType::CANCEL
        );
        assert_eq!(
            FrameType::from_bits(0x0A << 10).unwrap(),
            FrameType::PAYLOAD
        );
        assert_eq!(
            FrameType::from_bits(0x0B << 10).unwrap(),
            FrameType::ERROR
        );
        assert_eq!(
            FrameType::from_bits(0x0C << 10).unwrap(),
            FrameType::METADATA_PUSH
        );
        assert_eq!(
            FrameType::from_bits(0x0D << 10).unwrap(),
            FrameType::RESUME
        );
        assert_eq!(
            FrameType::from_bits(0x0E << 10).unwrap(),
            FrameType::RESUME_OK
        );
        assert_eq!(FrameType::from_bits(0x3F << 10).unwrap(), FrameType::EXT);
        assert!(FrameType::from_bits(0x00).is_none());
    }

    #[test]
    fn value() {
        assert_eq!(FrameType::SETUP.value(), 0x01);
    }

    #[test]
    fn bits() {
        assert_eq!(FrameType::SETUP.bits(), 0b0000_0100_0000_0000);
    }
}
