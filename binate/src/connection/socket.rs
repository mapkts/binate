use crate::connection::{
    ConnectionStatus, DuplexConnection, RequestCounter, StreamIdProvider,
};
use crate::error::Result;
use crate::error::Timeout as KeepaliveTimeout;
use crate::frame::{codec::*, Frame};
use crate::payload::Payload;
use crate::runtime;
use crate::types::{Subject, Subscription};
use crate::{Flux, Metadata, Mono, RSocket};

use dashmap::DashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tokio_stream::StreamExt;
use tracing::error;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Client,
    Server,
}

#[derive(Clone)]
struct RequestHanlder(Arc<RwLock<Box<dyn RSocket>>>);

#[derive(Clone)]
pub(crate) struct RSocketMachine {
    role: Role,
    stream_id: Arc<StreamIdProvider>,
    connection: Arc<Box<dyn DuplexConnection>>,
    request_handler: RequestHanlder,
    receivers: Arc<DashMap<u32, Box<dyn Subject<Item = Frame>>>>,
    subscriptions: Arc<DashMap<u32, Box<dyn Subscription>>>,
    request_n: Arc<RequestCounter>,
    chunk_payload: Option<usize>,
    keepalive_timeout: Duration,
    keepalive_last_received: Instant,
}

impl RSocketMachine {
    pub(crate) async fn new(
        role: Role,
        connection: Box<dyn DuplexConnection>,
        keepalive_timeout: Duration,
    ) -> RSocketMachine {
        let stream_id = match role {
            Role::Server => Arc::new(StreamIdProvider::new_for_server()),
            Role::Client => Arc::new(StreamIdProvider::new_for_client()),
        };

        let rsm = RSocketMachine {
            role,
            stream_id,
            connection: Arc::new(connection),
            request_handler: RequestHanlder(Arc::new(RwLock::new(Box::new(
                crate::rsocket::DummyRSocket,
            )))),
            receivers: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            request_n: Arc::new(RequestCounter::new(0)),
            chunk_payload: None,
            keepalive_timeout,
            keepalive_last_received: Instant::now(),
        };

        // Listens to the connection status.
        let mut cloned_rsm = rsm.clone();
        runtime::spawn(async move {
            while let Some(status) =
                cloned_rsm.connection.connection_status().next().await
            {
                match status {
                    ConnectionStatus::Closed => {
                        cloned_rsm.handle_transport_close();
                    }
                    ConnectionStatus::Error(err) => {
                        cloned_rsm.handle_error(&err);
                    }
                    _ => (),
                }
            }
        });

        let mut cloned_rsm = rsm.clone();
        runtime::spawn(async move {
            loop {
                tokio::time::sleep(cloned_rsm.keepalive_timeout).await;
                let now = Instant::now();
                if now - cloned_rsm.keepalive_last_received
                    > cloned_rsm.keepalive_timeout
                {
                    cloned_rsm.handle_connection_error(&KeepaliveTimeout);
                    break;
                }
            }
        });

        rsm
    }

    pub fn close(&mut self) {
        self.connection.close()
    }
}

impl RSocketMachine {
    fn handle_connection_error(&mut self, error: &impl fmt::Display) {
        self.handle_error(error);
        self.connection.close();
    }

    fn handle_error(&mut self, error: &impl fmt::Display) {
        error!("{}", error);
    }

    fn handle_transport_close(&mut self) {
        self.handle_error(&"connection was closed");
    }
}

impl RequestHanlder {
    pub(crate) async fn set_request_handler(&self, handler: Box<dyn RSocket>) {
        let mut wtr = self.0.write().await;
        *wtr = handler;
    }
}

impl RSocket for RSocketMachine {
    fn request_response(&self, _payload: Payload) -> Mono<Result<Payload>> {
        unimplemented!()
    }

    fn request_stream(&self, _payload: Payload) -> Flux<Result<Payload>> {
        unimplemented!()
    }

    fn request_channel(
        &self,
        _payloads: Flux<Result<Payload>>,
    ) -> Flux<Result<Payload>> {
        unimplemented!()
    }

    fn fire_and_forget(&self, payload: Payload) -> Result<()> {
        let stream_id = self.stream_id.next_stream_id(&self.receivers);

        if let Some(chunk_size) = self.chunk_payload {
            let chunks = payload.chunks(chunk_size);
            let chunks_len = chunks.len();
            for (idx, chunked) in chunks.enumerate() {
                let follows = idx + 1 != chunks_len;
                let frame = Frame::RequestFnf(RequestFnfFrame::new(
                    stream_id, follows, chunked,
                ));
                self.connection.send_and_forget(frame)?;
            }
        } else {
            let frame = Frame::RequestFnf(RequestFnfFrame::new(
                stream_id, false, payload,
            ));
            self.connection.send_and_forget(frame)?;
        }

        Ok(())
    }

    fn metadata_push(&self, metadata: Metadata) -> Mono<Result<()>> {
        let frame = Frame::MetadataPush(MetadataPushFrame::new(metadata));
        self.connection.send(frame)
    }
}
