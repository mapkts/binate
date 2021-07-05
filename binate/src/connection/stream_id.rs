use dashmap::DashMap;

cfg_not_loom! {
    use std::sync::atomic::{AtomicI32, Ordering};
}

cfg_loom! {
    use loom::sync::atomic::{AtomicI32, Ordering};
}

const STREAM_ID_MASK: i32 = 0x7FFFFFFF;

/// Thread safe stream ID provider.
#[derive(Debug)]
pub struct StreamIdSupplier(AtomicI32);

impl StreamIdSupplier {
    /// Create a `StreamIdSupplier` for the client side.
    pub fn new_for_client() -> StreamIdSupplier {
        let sid = AtomicI32::new(1);
        StreamIdSupplier(sid)
    }

    /// Create a `StreamIdSupplier` for the server side.
    pub fn new_for_server() -> StreamIdSupplier {
        let sid = AtomicI32::new(2);
        StreamIdSupplier(sid)
    }

    /// Returns the next available stream ID.
    pub fn next<T>(&self, store: &DashMap<u32, T>) -> u32 {
        let mut sid;
        loop {
            sid = (self.0.fetch_add(2, Ordering::Relaxed) & STREAM_ID_MASK)
                as u32;

            if !store.contains_key(&sid) {
                break;
            }
        }
        sid
    }

    // for testing only
    fn _new(init: i32) -> StreamIdSupplier {
        let sid = AtomicI32::new(init);
        StreamIdSupplier(sid)
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn assert_send_sync() {
        assert_send::<StreamIdSupplier>();
        assert_sync::<StreamIdSupplier>();
    }

    #[test]
    fn first_client_stream_id() {
        let store: DashMap<u32, ()> = DashMap::new();
        let gen = StreamIdSupplier::new_for_client();
        assert_eq!(gen.next(&store), 1);
        assert_eq!(gen.next(&store), 3);
    }

    #[test]
    fn first_server_stream_id() {
        let store: DashMap<u32, ()> = DashMap::new();
        let gen = StreamIdSupplier::new_for_server();
        assert_eq!(gen.next(&store), 2);
        assert_eq!(gen.next(&store), 4);
    }

    #[test]
    fn skip_existing_one() {
        let store: DashMap<u32, ()> = DashMap::new();
        store.insert(3, ());
        let gen = StreamIdSupplier::new_for_client();
        assert_eq!(gen.next(&store), 1);
        assert_eq!(gen.next(&store), 5);
    }

    #[test]
    fn wraps_around_on_overflow() {
        let store: DashMap<u32, ()> = DashMap::new();
        let gen = StreamIdSupplier::_new(i32::MAX);
        assert_eq!(gen.next(&store), STREAM_ID_MASK as u32);
        assert_eq!(gen.next(&store), 1);

        let store: DashMap<u32, ()> = DashMap::new();
        let gen = StreamIdSupplier::_new(-1);
        assert_eq!(gen.next(&store), STREAM_ID_MASK as u32);
        assert_eq!(gen.next(&store), 1);
    }
}

#[cfg(all(test, loom))]
mod tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn assert_thread_safe() {
        loom::model(|| {
            let gen = Arc::new(StreamIdSupplier::new_for_server());
            let store: Arc<DashMap<u32, ()>> = Arc::new(DashMap::new());
            store.insert(4, ());
            store.insert(8, ());

            let threads: Vec<_> = (0..2)
                .map(|_| {
                    let gen = gen.clone();
                    let store = store.clone();
                    loom::thread::spawn(move || {
                        gen.next(&store);
                    })
                })
                .collect();

            gen.next(&store);
            gen.next(&store);

            for th in threads {
                th.join().unwrap()
            }

            // (2 + 2 + 2 + 1) * 2
            assert_eq!(gen.next(&store), 14);
        })
    }
}
