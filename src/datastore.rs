use actix::prelude::*;
use chrono::prelude::*;
use std::collections::vec_deque::VecDeque;
use std::fmt;
use std::fmt::Debug;
use std::time::SystemTime;

struct HistoryBucket<D> {
    data: D,
    timestamp: SystemTime,
}

#[derive(Debug)]
struct DebugHistoryBucket<'a, D>
where
    D: Debug,
{
    data: &'a D,
    timestamp_millis: i64,
    datetime_utc: String,
}

impl<D> Debug for HistoryBucket<D>
where
    D: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let utc: DateTime<Utc> = DateTime::from(self.timestamp);
        let d = DebugHistoryBucket {
            data: &self.data,
            timestamp_millis: utc.timestamp_millis(),
            datetime_utc: utc.to_rfc3339(),
        };
        write!(f, "{:?}", d)
    }
}

pub struct DataStore<D>
where
    D: Debug,
{
    history: VecDeque<HistoryBucket<D>>,
    capacity: usize,
}

impl<D> DataStore<D>
where
    D: Debug,
{
    pub fn new(capacity: usize) -> Self {
        let capacity = match capacity {
            0 => 1,
            c => c,
        };
        DataStore {
            history: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    fn insert(&mut self, data: D) {
        if self.history.len() == self.capacity {
            self.history.pop_front();
        }
        self.history.push_back(HistoryBucket {
            data,
            timestamp: SystemTime::now(),
        });
    }
}

impl<D> Actor for DataStore<D>
where
    D: Debug + 'static,
{
    type Context = Context<Self>;
}

impl<D> Handler<D> for DataStore<D>
where
    D: Debug + 'static + Message<Result = ()>,
{
    type Result = ();

    fn handle(&mut self, msg: D, ctx: &mut Self::Context) -> Self::Result {
        self.insert(msg);
    }
}

pub struct DebugHistory;
impl Message for DebugHistory {
    type Result = String;
}
impl<D> Handler<DebugHistory> for DataStore<D>
where
    D: Debug + 'static,
{
    type Result = String;

    fn handle(&mut self, msg: DebugHistory, ctx: &mut Self::Context) -> Self::Result {
        format!("{:#?}", self.history)
    }
}
