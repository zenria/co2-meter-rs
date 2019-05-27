use actix::prelude::*;
use futures::{future, stream, Future, Stream};

pub trait BroadcastableMessage: Message<Result = ()> + Clone + Send + 'static {}

// auto impl
impl<M> BroadcastableMessage for M where M: Message<Result = ()> + Clone + Send + 'static {}

/// Broadcast a message to subscribers.
///
/// Fire & forget, if an error occurs it will be silently discarded
pub struct Broadcast<M: BroadcastableMessage> {
    subscribers: Vec<Recipient<M>>,
}

impl<M: BroadcastableMessage> Broadcast<M> {
    pub fn new() -> Self {
        Broadcast {
            subscribers: Vec::new(),
        }
    }
}

impl<M: BroadcastableMessage> Actor for Broadcast<M> {
    type Context = Context<Self>;
}

impl<M: BroadcastableMessage> Handler<M> for Broadcast<M> {
    type Result = ();

    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Self::Result {
        // Convert subscribers into a stream of future, resolves them in parallel using
        // buffer_unordered
        let futures: Vec<_> = self
            .subscribers
            .iter()
            .map(|subscriber| {
                subscriber
                    .clone()
                    .send(msg.clone())
                    .map_err(|e| error!("Unable to deliver message {}", e))
            })
            .collect();
        stream::iter_ok(futures)
            .buffer_unordered(self.subscribers.len())
            .fold((), |acc, x| future::ok(())) // consumer every element of the stream into nothing
            .into_actor(self)
            .spawn(ctx);
    }
}

#[derive(Message)]
pub struct RegisterRecipient<M: BroadcastableMessage>(pub Recipient<M>);

impl<M: BroadcastableMessage> Handler<RegisterRecipient<M>> for Broadcast<M> {
    type Result = ();

    fn handle(&mut self, msg: RegisterRecipient<M>, ctx: &mut Self::Context) -> Self::Result {
        self.subscribers.push(msg.0);
    }
}
