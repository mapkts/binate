use crate::frame::Frame;
use crate::Flux;

/// Represents a network connection over `RSocket` to send/receive data.
pub(crate) trait DuplexConnection: Send + Sync {
    /// Send a frame to the remote peer.
    ///
    /// Do nothing if the underlying connection is closed.
    fn send(&self, frame: Frame);

    /// Send a stream of frames to the remote peer.
    fn send_stream(&self, frames: Flux<Frame>);

    /// Returns a stream of frames received on this connection.
    fn recv(&self) -> Flux<Frame>;

    /// Returns whether the duplex connection respects frame boundaries.
    fn is_framed() -> bool;

    /// Open the underlying connection.
    fn connect(&self);

    /// Close the underlying connection.
    fn close(&self);
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
}
