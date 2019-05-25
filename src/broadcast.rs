use actix::prelude::*;

trait AsyncSubscriber<M>
where
    M: Message<Result = ()>,
    M: Clone,
    M: 'static,
    Self: Handler<M, Result = ()>,
    Self: Actor,
{
}

/// Broadcast a message to multiple actors
struct Broadcast<M>
where
    M: Message<Result = ()>, // defines all possible bounds straight from the first struct
    M: Clone,
    M: 'static,
{
    subscribers: Vec<Addr<Box<AsyncSubscriber<M>>>>,
}

impl<M> Actor for Broadcast<M>
where
    M: Message<Result = ()>,
    M: Clone,
    M: 'static,
{
    type Context = Context<Self>;
}
impl<M> Handler<M> for Broadcast<M>
where
    M: Message<Result = ()>,
    M: Clone,
    M: 'static,
{
    type Result = ();

    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}
