use std::collections::{BTreeMap, HashMap, HashSet};

use schedule::{Schedule, NodeService, Node};
use kk::logger;
use kk::random::{global_rng};
use kk::random::seq::sample_iter;
use kk::shield::{Shield, ShieldExt};


struct Running<'a> {
    total: u32,
    servers: HashMap<&'a str, u32>,
}

pub fn split_role_name(rname: &str) -> Option<(&str, &str)> {
    let mut iter = rname.splitn(3, "--");
    match (iter.next(), iter.next(), iter.next()) {
        (Some(proj), Some(group), None) => return Some((proj, group)),
        _ => {
            trace!("Role {:?} is skipped", rname);
            return None;
        }
    }
}

fn add_services<'a>(r: &mut Running<'a>, hosts: &'a HashSet<String>, n: u32) {
    // TODO(tailhook) this is super-lazy code
    let order = sample_iter(&mut global_rng(), hosts, n as usize)
        .unwrap_or_else(|v| v);
    for h in order.iter().cycle().take(n as usize) {
        *r.servers.entry(h).or_insert(0) += 1;
        r.total += 1;
    }
}

fn drop_services<'a>(r: &mut Running<'a>, hosts: &'a HashSet<String>, n: u32) {
    // TODO(tailhook) this is super-inefficient code
    let order = sample_iter(&mut global_rng(), hosts, n as usize)
        .unwrap_or_else(|v| v);
    for &h in order.iter().cycle() {
        if let Some(mut n) = r.servers.get_mut(&h[..]) {
            *n -= 1;
            if *n > 0 {
                r.total -= 1;
                continue;
            }
        } else {
            continue;
        }
        let n = r.servers.remove(&h[..]).expect("still not removed");
        debug_assert_eq!(n, 0);
        r.total -= 1;
    }
}

pub fn distribute(schedule: &mut Schedule, hosts: &HashSet<String>)
    -> BTreeMap<String, Shield<Node>>
{
    let mut r = HashMap::new();
    for (nname, mut node) in &mut schedule.nodes {
        let node = node.ensure_valid();
        for (rname, mut nrole) in &mut node.roles {
            let (pname, gname) = match split_role_name(rname) {
                Some(pair) => pair,
                None => continue,
            };
            let nrole = nrole.ensure_valid();
            for (sname, mut service) in &mut nrole.services {
                let service = service.ensure_valid();
                let mut rentry = r.entry((pname, gname, sname))
                    .or_insert_with(|| Running {
                        total: 0,
                        servers: HashMap::new(),
                    });
                rentry.total += service.instances;
                rentry.servers.insert(nname, service.instances);
            }
        }
    }
    let mut new_nodes: BTreeMap<String, Shield<Node>> = BTreeMap::new();
    for (pname, proj) in &schedule.projects {
        for (gname, group) in &proj.groups {
            let rname = format!("{}--{}", pname, gname);
            let _sub = logger::Sublogger::context(&rname);
            for (sname, service) in &group.services {
                let mut rentry = r.entry((pname, gname, sname))
                    .or_insert_with(|| Running {
                        total: 0,
                        servers: HashMap::new(),
                    });
                if rentry.total < service.instances {
                    let n = service.instances - rentry.total;
                    add_services(rentry, hosts, n);
                } else {
                    let n = rentry.total - service.instances;
                    drop_services(rentry, hosts, n);
                }
                if service.instances > 0 {
                    let img_opt = schedule.sources.get(&service.source)
                        .and_then(|src| {
                            src.deployments.get(&service.version)
                            .and_then(|ver| ver.containers
                                .iter()
                                .find(|&c| src.images.get(c)
                                    .and_then(
                                        |img| img.daemons.get(&service.config))
                                    .is_some()))
                        });
                    let img = if let Some(img) = img_opt {
                        format!("{}/{}/{}",
                            service.source,
                            img.splitn(2, ".").next().unwrap(),
                            img)
                    } else {
                        debug!("No image found for service {:?}", service);
                        continue;
                    };
                    for h in hosts {
                        let n = rentry.servers.get(&h[..])
                            .map(|&x| x).unwrap_or(0);
                        if n != 0 {
                            let nrole = new_nodes.entry(h.to_string())
                                .ensure_valid()
                                .roles.entry(rname.clone())
                                .ensure_valid();
                            nrole.version = service.version.clone();
                            nrole.template = Some("service".into());
                            nrole.services.insert(sname.clone(), NodeService {
                                image: img.clone(),
                                config: service.config.clone(),
                                instances: n,
                            }.into());
                        }
                    }
                }
            }
        }
    }
    return new_nodes;
}
