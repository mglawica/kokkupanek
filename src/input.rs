use std::net::SocketAddr;
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map;
use std::time::SystemTime;


#[derive(Clone, Debug, Deserialize)]
pub struct Peer {
    pub addr: Option<SocketAddr>,
    pub name: String,
    pub hostname: String,
    #[serde(with="::serde_millis")]
    pub known_since: SystemTime,
    #[serde(with="::serde_millis")]
    pub last_report_direct: Option<SystemTime>,
}

#[derive(Deserialize, Debug)]
pub struct GenericInput<A, S, R> {
    #[serde(with="::serde_millis")]
    pub now: SystemTime,
    pub current_host: String,
    pub current_id: String, //Id,
    pub parents: Vec<S>,
    pub actions: BTreeMap<u64, A>,
    pub runtime: R,
    pub peers: HashMap<String, Peer>,
}

/// Iterator over all hosts in the cluster
///
/// The `peers` map contains only "other" peers, i.e. excludes current host.
/// This iterator includes all the hostnames;
#[derive(Debug, Clone)]
pub struct Hosts<'a>(Option<&'a str>, hash_map::Iter<'a, String, Peer>);

pub trait Input {
    fn now(&self) -> SystemTime;
}

impl<A, S, R> Input for GenericInput<A, S, R> {
    fn now(&self) -> SystemTime {
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

impl<'a> Iterator for Hosts<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        self.0.take().or_else(|| self.1.next().map(|(_, n)| &n.hostname[..]))
    }
}

impl<'a> IntoIterator for &'a Hosts<'a> {
    type Item = &'a str;
    type IntoIter = Hosts<'a>;
    fn into_iter(self) -> Hosts<'a> {
        self.clone()
    }
}

impl<A, S, R> GenericInput<A, S, R> {
    pub fn hosts(&self) -> Hosts {
        Hosts(Some(&self.current_host), self.peers.iter())
    }
}
