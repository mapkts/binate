//! Implementation of RSocket frame types and frame codec.
use super::*;

// In order for macros to be visible to submodules, we need to declear them first.
macro_rules! debug_assert_max_u31 {
    ($($var:ident),+) => {
        $(debug_assert!($var <= MAX_U31, concat!(stringify!($var), " MUST be <= MAX_U31 (0x7FFFFFFF)"));)+
    }
}

macro_rules! debug_assert_max_u63 {
    ($($var:ident),+) => {
        $(debug_assert!($var <= MAX_U63, concat!(stringify!($var), " MUST be <= MAX_U63 (0x7FFF_FFFF_FFFF_FFFF)"));)+
    }
}

macro_rules! debug_assert_non_zero {
    ($($var:ident),+) => {
        $(debug_assert!($var != 0, concat!(stringify!($var), " MUST be non-zero"));)+
    }
}

mod cancel;
mod error;
mod keepalive;
mod lease;
mod metadata_push;
mod payload;
mod request_channel;
mod request_fnf;
mod request_n;
mod request_response;
mod request_stream;
mod resume;
mod resume_ok;
mod setup;

pub use self::cancel::CancelFrame;
pub use self::error::ErrorFrame;
pub use self::keepalive::KeepaliveFrame;
pub use self::lease::LeaseFrame;
pub use self::metadata_push::MetadataPushFrame;
pub use self::payload::PayloadFrame;
pub use self::request_channel::RequestChannelFrame;
pub use self::request_fnf::RequestFnfFrame;
pub use self::request_n::RequestNFrame;
pub use self::request_response::RequestResponseFrame;
pub use self::request_stream::RequestStreamFrame;
pub use self::resume::ResumeFrame;
pub use self::resume_ok::ResumeOkFrame;
pub use self::setup::{SetupFrame, SetupFrameBuilder};
