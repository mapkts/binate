pub(crate) trait Subscriber {
    type Item;

    fn on_subscribe(&mut self, channel: usize) -> crate::Result<()>;

    fn on_next(&mut self, item: Self::Item) -> crate::Result<()>;

    fn on_error(&mut self, err: crate::Error);

    fn on_complete(&mut self) -> crate::Result<()>;
}

pub(crate) trait Publisher<'a> {
    type Output;

    fn subscribe<F, S>(&mut self, f: F) -> crate::Result<()>
    where
        F: FnOnce(Box<S>),
        S: Subscriber<Item = Self::Output> + 'a;
}

pub(crate) trait Subscription: Send + Sync + 'static {
    fn cancel(&mut self) -> crate::Result<()>;
}

pub(crate) trait Subject: Send + Sync + 'static {
    type Item;

    fn on_next(&mut self, item: Self::Item) -> crate::Result<()>;

    fn on_error(&mut self, err: crate::Error);

    fn on_complete(&mut self) -> crate::Result<()>;
}
