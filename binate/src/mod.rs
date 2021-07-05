//! Implementation of the RSocket protocol.
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

mod socket;
pub use socket::{Flux, Mono, RSocket};
