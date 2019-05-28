use crate::broadcast::Broadcast;
use crate::broadcast::BroadcastableMessage;
use crate::mqtt::MqttData;
use actix::prelude::*;
use futures::future::Future;
use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;

pub mod mh_z19;

// Send this message to tell the sensor to read data
pub struct ReadMessage<R, E>
where
    R: 'static,
    E: 'static,
{
    _response: PhantomData<R>,
    _error: PhantomData<E>,
}

impl<R, E> ReadMessage<R, E>
where
    R: 'static,
    E: 'static,
{
    fn new() -> Self {
        ReadMessage {
            _error: PhantomData,
            _response: PhantomData,
        }
    }
}

impl<R, E> Message for ReadMessage<R, E> {
    type Result = Result<R, E>;
}
/// Actor resposible for sending read command to sensor and forward results to the DataStore
pub struct SensorReader<A, R, E>
where
    A: Actor<Context = SyncContext<A>>,
    A: Handler<ReadMessage<R, E>>,
    E: Error + 'static + Send,
    R: BroadcastableMessage + Debug + MqttData,
{
    sensor: Addr<A>,
    read_interval: Duration,
    _error: PhantomData<E>,
    _response: PhantomData<R>,
}
impl<A, R, E> SensorReader<A, R, E>
where
    A: Actor<Context = SyncContext<A>>,
    A: Handler<ReadMessage<R, E>>,
    E: Error + 'static + Send,
    R: BroadcastableMessage + Debug + MqttData,
{
    pub fn new(sensor: Addr<A>, read_interval: Duration) -> Self {
        SensorReader {
            sensor,
            read_interval,
            _error: PhantomData,
            _response: PhantomData,
        }
    }

    fn send_to_registered_listeners(
        &self,
        data: R,
    ) -> impl ActorFuture<Item = (), Error = (), Actor = Self> {
        System::current()
            .registry()
            .get::<Broadcast<R>>()
            .send(data)
            .map_err(|e| error!("Unable to send data to sensor services {}", e))
            .into_actor(self)
    }

    fn read_sensor(&mut self, ctx: &mut Context<SensorReader<A, R, E>>) {
        self.sensor
            .send(ReadMessage::new())
            .into_actor(self)
            .map_err(|e, _act, _ctx| {
                error!("Unable to send ReadMessage to sensor actor {}", e);
            })
            .and_then(|result, reader, ctx| match result {
                // unwrap the error
                Ok(r) => {
                    debug!("Read {:?}", r);
                    actix::fut::ok(r)
                }
                Err(e) => {
                    error!("Error reading the sensor: {}", e);
                    actix::fut::err(())
                }
            })
            .and_then(|data: R, reader: &mut SensorReader<A, R, E>, ctx| {
                reader.send_to_registered_listeners(data)
            })
            .spawn(ctx);
    }
}

impl<A, R, E> Actor for SensorReader<A, R, E>
where
    A: Actor<Context = SyncContext<A>>,
    A: Handler<ReadMessage<R, E>>,
    E: Error + 'static + Send,
    R: BroadcastableMessage + Debug + MqttData,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(self.read_interval, |sensor_reader, ctx| {
            debug!("Sending read message!");
            sensor_reader.read_sensor(ctx);
        });
        self.read_sensor(ctx);
    }
}
