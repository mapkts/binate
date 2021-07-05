use super::frame::Payload;
use super::Result;
use std::future::Future;
use std::pin::Pin;
use tokio_stream::Stream;

/// A stream that emits a value exactly once.
pub type Mono<T> = Pin<Box<dyn Send + Sync + Future<Output = T>>>;

/// A stream of values that produced asynchronously.
pub type Flux<T> = Pin<Box<dyn Send + Sync + Stream<Item = T>>>;

/// A trait that represents a Reactive Socket.
pub trait RSocket: Send + Sync {
    /// Request-Response interaction model of RSocket.
    fn request_response(&self, payload: Payload) -> Mono<Result<Payload>>;

    /// Request-Stream interaction model of RSocket.
    fn request_stream(&self, payload: Payload) -> Flux<Result<Payload>>;

    /// Request-Channel interaction model of RSocket.
    fn request_channel(
        &self,
        payloads: Flux<Result<Payload>>,
    ) -> Flux<Result<Payload>>;

    /// Fire-and-Forget interaction model of RSocket.
    fn fire_and_forget(&self, payload: Payload) -> Mono<Result<()>>;

    /// Metadata-Push interaction model of RSocket.
    fn metadata_push(&self, payload: Payload) -> Mono<Result<()>>;
}
