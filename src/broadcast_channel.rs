use crossbeam::channel::RecvTimeoutError;
use crossbeam::channel::{Receiver, RecvError, Sender};
use crossbeam::queue::SegQueue;
use std::time::Duration;

pub struct OneShotBroadcastReceiver<T> {
    inner: Receiver<T>,
    senders: SegQueue<Sender<T>>,
}

impl<T: Clone> OneShotBroadcastReceiver<T> {
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let ret = self.inner.recv_timeout(timeout)?;

        loop {
            match self.senders.pop() {
                Ok(sender) => {
                    // broadcast ignoring unconnected or full subscribed receiver
                    let _r = sender.try_send(ret.clone());
                }
                Err(_) => break,
            }
        }
        Ok(ret)
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        let ret = self.inner.recv()?;

        loop {
            match self.senders.pop() {
                Ok(sender) => {
                    // broadcast ignoring unconnected or full subscribed receiver
                    let _r = sender.try_send(ret.clone());
                }
                Err(_) => break,
            }
        }
        Ok(ret)
    }

    pub fn oneshot_receiver(&self) -> Receiver<T> {
        let (sender, receiver) = crossbeam::channel::bounded(0);
        self.senders.push(sender);
        receiver
    }
}

impl<T> From<Receiver<T>> for OneShotBroadcastReceiver<T> {
    fn from(inner: Receiver<T>) -> Self {
        Self {
            inner,
            senders: SegQueue::new(),
        }
    }
}

#[cfg(test)]
#[test]
fn test() {
    let (s, r) = crossbeam::channel::unbounded();

    let r = OneShotBroadcastReceiver::from(r);

    s.send(0).unwrap();
    assert_eq!(0, r.recv().unwrap());

    let r2 = r.oneshot_receiver();
    assert_eq!(
        1,
        crossbeam::scope(|scope| {
            scope.spawn(|_| {
                assert_eq!(1, r2.recv_timeout(Duration::from_secs(10)).unwrap());
            });

            scope.spawn(|_| s.send(1));
            r.recv().unwrap()
        })
        .unwrap()
    );
}
