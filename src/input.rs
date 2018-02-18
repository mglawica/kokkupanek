use std::net::SocketAddr;
use std::collections::{BTreeMap, HashMap};

use timestamp::Timestamp;


#[derive(Clone, Debug, Deserialize)]
pub struct Peer {
    pub addr: Option<SocketAddr>,
    pub name: String,
    pub hostname: String,
    pub known_since: Timestamp,
    pub last_report_direct: Option<Timestamp>,
}

#[derive(Deserialize, Debug)]
pub struct GenericInput<A, S, R> {
    pub now: Timestamp,
    pub current_host: String,
    pub current_id: String, //Id,
    pub parents: Vec<S>,
    pub actions: BTreeMap<u64, A>,
    pub runtime: R,
    pub peers: HashMap<String, Peer>,
}

pub trait Input {
    fn now(&self) -> Timestamp;
}

impl<A, S, R> Input for GenericInput<A, S, R> {
    fn now(&self) -> Timestamp {
        self.now
    }
}

pub trait Schedule {
    fn new() -> Self;
    fn merge(&mut self, other: Self);
    fn from_parents<I>(parents: I) -> Self
        where I: IntoIterator<Item=Self>,
              Self: Sized,
    {
        let mut s = Self::new();
        for next in parents.into_iter() {
            s.merge(next);
        }
        return s;
    }
}
