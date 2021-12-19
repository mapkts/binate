//! Implementation of the RSocket protocol.
#![warn(
    rust_2018_idioms,
    missing_docs,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(dead_code)]

#[macro_use]
extern crate bitflags;

#[macro_use]
#[doc(hidden)]
pub(crate) mod macros;
pub(crate) mod test_helpers;

mod consts;
mod error;
mod payload;
mod rsocket;
mod runtime;
mod types;

pub mod connection;
pub mod mimetype;
pub mod prelude;

cfg_doc! {
    #[feature = "frame"]
    pub mod frame;
}

cfg_not! {
    #[feature = "frame"]
    mod frame;
}

pub use self::error::{Code, Error, Result};
pub use self::payload::{Data, Metadata, Payload, PayloadBuilder};
pub use self::rsocket::{Flux, Mono, RSocket};
