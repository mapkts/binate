use crate::payload::Payload;
use crate::Result;

use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;
use tokio_stream::Stream;

/// A stream that emits a value exactly once.
pub type Mono<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// A stream of values that produced asynchronously.
pub type Flux<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

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
    fn fire_and_forget(&self, payload: Payload) -> Result<()>;

    /// Metadata-Push interaction model of RSocket.
    fn metadata_push(&self, metadata: Bytes) -> Mono<Result<()>>;
}

#[derive(Clone)]
pub(crate) struct DummyRSocket;

impl RSocket for DummyRSocket {
    /// Request-Response interaction model of RSocket.
    fn request_response(&self, _payload: Payload) -> Mono<Result<Payload>> {
        unimplemented!("dummy rsocket");
    }

    /// Request-Stream interaction model of RSocket.
    fn request_stream(&self, _payload: Payload) -> Flux<Result<Payload>> {
        unimplemented!("dummy rsocket");
    }

    /// Request-Channel interaction model of RSocket.
    fn request_channel(
        &self,
        _payloads: Flux<Result<Payload>>,
    ) -> Flux<Result<Payload>> {
        unimplemented!("dummy rsocket");
    }

    /// Fire-and-Forget interaction model of RSocket.
    fn fire_and_forget(&self, _payload: Payload) -> Result<()> {
        unimplemented!("dummy rsocket");
    }

    /// Metadata-Push interaction model of RSocket.
    fn metadata_push(&self, _metadata: Bytes) -> Mono<Result<()>> {
        unimplemented!("dummy rsocket");
    }
}
