use crate::error::Result;
use crate::frame::Frame;
use crate::payload::Payload;
use crate::{Flux, Mono};

use bytes::Bytes;

/// Represents a network connection over `RSocket` to send/receive data.
pub trait DuplexConnection: Send + Sync {
    /// Send a frame to the remote peer.
    ///
    /// Do nothing if the underlying connection is closed.
    fn send(&self, frame: Frame) -> Mono<Result<()>>;

    /// Similar to [`send`], but doesn't wait for response.
    fn send_and_forget(&self, frame: Frame) -> Result<()>;

    /// Send a stream of frames to the remote peer.
    ///
    /// Do nothing if the underlying connection is closed.
    fn send_stream(&self, frames: Flux<Frame>);

    /// Returns a stream of frames received on this connection.
    fn receive(&self) -> Flux<Frame>;

    /// Open the underlying connection.
    fn connect(&self);

    /// Close the underlying connection.
    fn close(&self);

    /// Returns a stream that immediately publishes the currrent connection status and thereafter
    /// updates as it changes.
    fn connection_status(&self) -> Flux<ConnectionStatus>;
}

/// Describes connection status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// No connection established or pending.
    Unconnected,
    /// `connect()` is called but a connection is not yet established.
    Connecting,
    /// Connection is established.
    Connected,
    /// Connection has been closed via `close()`.
    Closed,
    /// Connection has been closed for any other reason.
    Error(String),
}

/// Represents a server that accepts connections and turns them into `DuplexConnection`.
pub(crate) trait ConnectionAcceptor {
    /// Allocate required resources and begin listening for new connections.
    ///
    /// This can only be called once.
    fn start<F>(&self, on_accept: F)
    where
        F: FnOnce(&dyn DuplexConnection);

    /// Stop listening for new connections.
    ///
    /// This can only be called once.
    fn stop(&self);

    /// Returns the port the acceptor is listening to.
    fn listening_port(&self) -> usize;
}

/// Represents a responder that handles requests on an RSocket connection.
pub(crate) trait RSocketResponder {
    fn handle_request_response(
        &self,
        stream_id: u32,
        payload: Payload,
    ) -> Mono<Payload>;

    fn handle_request_stream(
        &self,
        stream_id: u32,
        payload: Payload,
    ) -> Flux<Payload>;

    fn handle_request_channel(
        &self,
        stream_id: u32,
        payload: Flux<Payload>,
    ) -> Flux<Payload>;

    fn handle_fire_and_forget(&self, stream_id: u32, payload: Payload);

    fn handle_metadata_push(&self, stream_id: u32, metadata: Bytes);
}
