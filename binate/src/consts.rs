use std::time::Duration;

/// Default value of the time between KEEPALIVE frames that the client will send.
pub const DEFAULT_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// Default value of the time that a client will allow a server to not respond to
/// a KEEPALIVE before it is assumed to be dead.
pub const DEFAULT_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60);
