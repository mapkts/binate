//! Payload of frame.
//!
//! Payload can be distinguished into two types: `Data` and `Metadata`. The distinction between
//! the types in an application is left to the application.
use crate::frame::Encode;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::cmp;

/// The data field of a `Payload`.
pub type Data = Bytes;

/// The meatadata field of a `Payload`.
pub type Metadata = Bytes;

/// Payload of a RSocket frame.
///
/// # Examples
///
/// ```
/// use binate::prelude::*;
///
/// let payload = Payload::builder().set_data("data").set_metadata("metadata").build();
/// assert_eq!(payload.data().unwrap(), "data");
/// assert_eq!(payload.metadata().unwrap(), "metadata");
/// ```
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Payload {
    pub(crate) metadata: Option<Metadata>,
    pub(crate) data: Option<Data>,
}

impl Payload {
    /// Constructs a payload with the given `data` and `metadata`.
    pub(crate) fn new(metadata: Option<Metadata>, data: Option<Data>) -> Self {
        Payload { metadata, data }
    }

    /// Returns a payload builder.
    pub fn builder() -> PayloadBuilder {
        PayloadBuilder::new()
    }

    /// Returns the number of bytes in this payload.
    pub fn len(&self) -> usize {
        let mut len = 0;
        if let Some(metadata) = &self.metadata {
            len += metadata.len();
        }
        if let Some(data) = &self.data {
            len += data.len();
        }
        len
    }

    /// Returns true if this payload is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the `data` part of this payload.
    pub fn data(&self) -> Option<&Data> {
        self.data.as_ref()
    }

    /// Returns the `metadata` part of this payload.
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Returns the `data` part of this payload in UTF-8 format, if the `data` is valid UTF-8.
    pub fn data_utf8(&self) -> Option<&str> {
        if let Some(ref data) = self.data {
            std::str::from_utf8(data).ok()
        } else {
            None
        }
    }

    /// Returns the `metadata` part of this payload in UTF-8 format, if the `metadata` is valid UTF-8.
    pub fn metadata_utf8(&self) -> Option<&str> {
        if let Some(ref metadata) = self.metadata {
            std::str::from_utf8(metadata).ok()
        } else {
            None
        }
    }

    /// Returns true if this payload contains `data`.
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    /// Returns true if this payload contains `metadata`.
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    /// Splits the payload into `Metadata` and `Data`.
    pub fn split(self) -> (Option<Metadata>, Option<Data>) {
        (self.metadata, self.data)
    }

    /// Returns a consuming iterator that yields `mtu` bytes of the payload at a time (both
    /// `metadata` and `data` are chunked by `mtu`).
    ///
    /// If `mtu` does not divide the the `metadata` and `data` of the payload, then the last chunk
    /// will not have length `mtu`.
    ///
    /// # Panics
    ///
    /// This function panics if the given `chunk_size` is `0`.
    pub fn chunks(self, chunk_size: usize) -> PayloadChunks {
        assert!(chunk_size != 0);
        let (metadata, data) = self.split();
        PayloadChunks { chunk_size, metadata, data }
    }
}

/// Construct a `Payload` with optional `Data` and/or `Metadata`.
///
/// # Examples
///
/// ```
/// use binate::prelude::*;
///
/// let payload = PayloadBuilder::new().set_data("data").set_metadata("metadata").build();
/// assert_eq!(payload.data().unwrap(), "data");
/// assert_eq!(payload.metadata().unwrap(), "metadata");
/// ```
#[derive(Debug)]
pub struct PayloadBuilder(Payload);

impl PayloadBuilder {
    /// Create a new `PayloadBuilder`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let payload = Payload::default();
        PayloadBuilder(payload)
    }

    /// Sets the data of the `Payload` to build.
    pub fn set_data<T>(mut self, data: T) -> Self
    where
        T: Into<Bytes>,
    {
        self.0.data = Some(data.into());
        self
    }

    /// Sets the metadata of the `Payload` to build.
    pub fn set_metadata<T>(mut self, metadata: T) -> Self
    where
        T: Into<Bytes>,
    {
        self.0.metadata = Some(metadata.into());
        self
    }

    /// Returns the configured `Payload`.
    pub fn build(self) -> Payload {
        self.0
    }
}

impl Encode for Payload {
    fn encode(&self, buf: &mut BytesMut) {
        // Metadata is always put before data.
        if let Some(meta) = &self.metadata {
            buf.put_slice(meta);
        }
        if let Some(data) = &self.data {
            buf.put_slice(data);
        }
    }

    fn len(&self) -> usize {
        let mut len = 0;
        if let Some(metadata) = &self.metadata {
            len += metadata.len();
        }
        if let Some(data) = &self.data {
            len += data.len();
        }
        len
    }
}

/// An iterator that yields chunked payload.
#[derive(Debug)]
pub struct PayloadChunks {
    chunk_size: usize,
    metadata: Option<Bytes>,
    data: Option<Bytes>,
}

impl PayloadChunks {
    /// Returns the number of chunks in this `PayloadChunks`.
    #[inline]
    pub fn len(&self) -> usize {
        let metadata_len =
            self.metadata.as_ref().map(|bytes| bytes.len()).unwrap_or(0);
        let data_len =
            self.data.as_ref().map(|bytes| bytes.len()).unwrap_or(0);
        let len1 = metadata_len as f32 / self.chunk_size as f32;
        let len2 = data_len as f32 / self.chunk_size as f32;

        cmp::max(len1.ceil() as usize, len2.ceil() as usize)
    }

    /// Returns true if this `PayloadChunks` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Iterator for PayloadChunks {
    type Item = Payload;

    fn next(&mut self) -> Option<Self::Item> {
        if self.metadata.is_none() && self.data.is_none() {
            return None;
        }

        let mut meta = None;
        let mut data = None;
        if let Some(m) = &mut self.metadata {
            let len = m.remaining();
            if self.chunk_size < len {
                meta = Some(m.split_to(self.chunk_size));
            } else {
                meta = self.metadata.take();
            }
        }
        if let Some(d) = &mut self.data {
            let len = d.remaining();
            if self.chunk_size < len {
                data = Some(d.split_to(self.chunk_size));
            } else {
                data = self.data.take();
            }
        }
        Some(Payload::new(meta, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let payload = Payload::builder()
            .set_metadata("metadata")
            .set_data("data payload")
            .build();

        let mut iter = payload.chunks(4);
        assert_eq!(
            iter.next(),
            Some(
                Payload::builder()
                    .set_metadata("meta")
                    .set_data("data")
                    .build()
            )
        );
        assert_eq!(
            iter.next(),
            Some(
                Payload::builder()
                    .set_metadata("data")
                    .set_data(" pay")
                    .build()
            )
        );
        assert_eq!(
            iter.next(),
            Some(Payload::builder().set_data("load").build())
        );
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn payload_chunks_len() {
        let payload = Payload::builder().build();
        assert_eq!(payload.chunks(1).len(), 0);

        let payload = Payload::builder()
            .set_metadata("metadata")
            .set_data("data payload")
            .build();
        assert_eq!(payload.clone().chunks(13).len(), 1);
        assert_eq!(payload.clone().chunks(12).len(), 1);
        assert_eq!(payload.clone().chunks(11).len(), 2);
        assert_eq!(payload.clone().chunks(10).len(), 2);
        assert_eq!(payload.clone().chunks(9).len(), 2);
        assert_eq!(payload.clone().chunks(8).len(), 2);
        assert_eq!(payload.clone().chunks(7).len(), 2);
        assert_eq!(payload.clone().chunks(6).len(), 2);
        assert_eq!(payload.clone().chunks(5).len(), 3);
        assert_eq!(payload.clone().chunks(4).len(), 3);
        assert_eq!(payload.clone().chunks(3).len(), 4);
        assert_eq!(payload.clone().chunks(2).len(), 6);
        assert_eq!(payload.chunks(1).len(), 12);
    }
}
