#![allow(proc_macro_derive_resolution_fallback)]

extern crate chrono;
extern crate futures;
extern crate failure;
extern crate env_logger;
extern crate futures_cpupool;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_qs;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate config as config_crate;
extern crate gelf;
extern crate simplelog;
extern crate base64;
extern crate hyper_tls;
extern crate num;
extern crate regex;
#[macro_use]
extern crate sentry;
extern crate tokio;
extern crate tokio_core;
extern crate uuid;

#[macro_use]
mod macros;
mod api;
mod client;
mod config;
mod logger;
mod models;
mod prelude;
mod sentry_integration;
mod utils;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use futures::future::Either;
use futures::{future, Future, Stream};
use tokio::timer::Interval;

use client::{
    BitcoinClient, BitcoinClientImpl, BlockchainInfoClient, BlockchainInfoClientImpl, HttpClientImpl, OpsGenieClient, OpsGenieClientImpl,
};
use models::*;

pub fn hello() {
    println!("Hello world");
}

pub fn print_config() {
    println!("Parsed config: {:?}", get_config());
}

pub fn start_server() {
    let config = get_config();
    // Prepare sentry integration
    let _sentry = sentry_integration::init(config.sentry.as_ref());
    // Prepare logger
    logger::init(&config);
    // Prepare nodes
    let nodes = config.to_nodes();
    let nodes = Arc::new(Mutex::new(nodes));
    let interval = Duration::from_secs(config.healthcheck.timeout);
    let quarantine_time = chrono::Duration::seconds(config.healthcheck.quarantine);
    let client = HttpClientImpl::new(&config);
    let nodes_clone = nodes.clone();
    let config_clone = config.clone();

    thread::spawn(move || {
        let mut core = tokio_core::reactor::Core::new().unwrap();
        let opsgenie_client = OpsGenieClientImpl::new(&config, client.clone());
        let healtcheck_client = BlockchainInfoClientImpl::new(&config, client.clone());
        let client = Arc::new(client);
        let nodes_clone2 = nodes_clone.clone();
        core.run(
            Interval::new(Instant::now(), interval)
                .map_err(|e| {
                    error!("Error creating interval {}", e);
                })
                .fold(nodes_clone, |nodes_clone, _| {
                    info!("Started healthcheck");
                    healtcheck_client
                        .get_block_count()
                        .then(|r| {
                            let (bitcoin_client, url, i) = {
                                let mut nodes_ = nodes_clone2.lock().unwrap();

                                let now = chrono::Utc::now().naive_utc();

                                // recovering nodes from quarantine
                                let quarantine_nodes: Vec<usize> = nodes_
                                    .iter()
                                    .filter_map(|(i, n)| {
                                        if let Quarantine::Yes(t) = n.quarantine {
                                            if (now - t) > quarantine_time {
                                                Some(*i)
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                for i in quarantine_nodes {
                                    let q_n = nodes_.get_mut(&i).expect("Can not find node in btreemap");
                                    q_n.quarantine = Quarantine::No;
                                }

                                //searching for first node not in quarantine
                                let node = nodes_.iter().filter(|(_, n)| n.quarantine == Quarantine::No).nth(0);

                                //if all nodes are in quarantine - take first
                                let (i, node) = node
                                    .unwrap_or_else(|| nodes_.get(&0).map(|n| (&0usize, n)).expect("There is no nodes defined in config"));

                                let url = node.url.clone();

                                let client =
                                    BitcoinClientImpl::new(client.clone(), node.url.clone(), node.user.clone(), node.password.clone());

                                (client, url, *i)
                            };

                            let nodes_clone3 = nodes_clone2.clone();

                            let opsgenie_client = opsgenie_client.clone();
                            match r {
                                Ok(count) => Either::A(bitcoin_client.get_last_block().then(move |bl_count| {
                                    match bl_count {
                                        Ok(bl_count) => Either::A({
                                            if count.checked_sub(bl_count).unwrap_or_else(|| u64::max_value()) > 1 {
                                                let mut nodes = nodes_clone3.lock().unwrap();
                                                let n = nodes.get_mut(&i).expect("Can not find node to compare in btreemap");
                                                n.quarantine = Quarantine::Yes(chrono::Utc::now().naive_utc());
                                                Either::A(
                                                    opsgenie_client
                                                        .notify(format!("Bitcoin node {} delay from blockchain exceeded limit.", url))
                                                        .map_err(|_| ()),
                                                )
                                            } else {
                                                Either::B(future::ok(()))
                                            }
                                        }),
                                        Err(e) => Either::B(
                                            opsgenie_client
                                                .notify(format!("Couldn't get last block from bitcoin node {} - {}", url, e))
                                                .map_err(|_| ()),
                                        ),
                                    }
                                })),
                                Err(e) => Either::B(
                                    opsgenie_client
                                        .notify(format!("Couldn't get last block from blockchain info - {}", e))
                                        .map_err(|_| ()),
                                ),
                            }
                        })
                        .then(move |_| future::ok(nodes_clone))
                })
                .map(|_| ()),
        )
    });

    // Start server
    api::start_server(config_clone, nodes);
}

fn get_config() -> config::Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
