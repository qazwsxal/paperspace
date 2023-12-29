use futures::stream::{self, Iter, Stream};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Response {
    Value(i64),
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Request {
    Increment(),
    Decrement(),
    Reset(),
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Counter {
    pub value: i64,
}

impl State for Counter {
    fn update(&mut self, req: Request) -> impl Stream<Item = Response> {
        match req {
            Request::Increment() => self.value += 1,
            Request::Decrement() => self.value -= 1,
            Request::Reset() => self.value = 0,
        };
        stream::iter(vec![Response::Value(self.value)])
    }

    fn dump(&self) -> Vec<Response> {
        vec![Response::Value(self.value)]
    }
}

pub trait State {
    fn update(&mut self, req: Request) -> impl Stream<Item = Response>;

    fn dump(&self) -> Vec<Response>;
}
