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

mod error;
pub use error::{Code, Error, Result};

mod rsocket;
pub use rsocket::{Flux, Mono, RSocket};

mod payload;
