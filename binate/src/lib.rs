//! Implementation of the RSocket protocol.
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate bitflags;

#[macro_use]
#[doc(hidden)]
pub mod macros;

pub mod connection;

cfg_frame! {
    pub mod frame;
}

cfg_not_frame! {
    mod frame;
}

pub mod mimetype;
pub mod prelude;

mod consts;
mod error;
mod payload;
mod rsocket;
mod runtime;
mod types;

pub use self::error::{Code, Error, Result};
pub use self::payload::{Data, Metadata, Payload, PayloadBuilder};
pub use self::rsocket::{Flux, Mono, RSocket};
