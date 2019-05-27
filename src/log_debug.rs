use actix::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct LogDebug<D> {
    _phantom: PhantomData<D>,
}
impl<D> LogDebug<D> {
    pub fn new() -> Self {
        LogDebug {
            _phantom: PhantomData,
        }
    }
}

impl<D: 'static> Actor for LogDebug<D> {
    type Context = Context<Self>;
}

impl<D: 'static + Message<Result = ()> + Debug> Handler<D> for LogDebug<D> {
    type Result = ();

    fn handle(&mut self, msg: D, ctx: &mut Self::Context) -> Self::Result {
        debug!("Received some data: {:?}", msg);
    }
}
