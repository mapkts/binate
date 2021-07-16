use std::future::Future;
use tokio::task::JoinHandle;

/// Spawns a new asynchronous task using tokio runtime, returning a `JoinHandle` for it.
pub fn spawn<T>(future: T) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    tokio::spawn(future)
}
