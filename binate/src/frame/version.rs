use super::*;
use bytes::{BufMut, BytesMut};
use std::cmp::Ordering;
use std::fmt;

/// Version number of the RSocket protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    major: u16,
    minor: u16,
}

impl Version {
    /// Builds the protocol version with the given `major` and `minor` number.
    pub fn new(major: u16, minor: u16) -> Self {
        Version { major, minor }
    }

    /// Returns the `major` number of this version.
    pub fn major(self) -> u16 {
        self.major
    }

    /// Returns the `minor` number of this version.
    pub fn minor(self) -> u16 {
        self.minor
    }
}

impl Encode for Version {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.major);
        buf.put_u16(self.minor);
    }

    fn len(&self) -> usize {
        4
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_major = self.major.cmp(&other.major);
        if cmp_major != Ordering::Equal {
            return cmp_major;
        }
        self.minor.cmp(&other.minor)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Default for Version {
    fn default() -> Version {
        Version::new(1, 0)
    }
}
