use failure::Fail;
use futures::prelude::*;

use super::super::utils::parse_body;
use super::Context;
use super::ControllerFuture;
use super::ErrorKind;
use client::{BitcoinClient, BitcoinClientImpl};
use models::*;

pub fn proxy(ctx: &Context) -> ControllerFuture {
    let client = {
        let mut nodes_ = ctx.nodes.lock().unwrap();
        //searching for main node not in quarantine
        let node = nodes_.values().filter(|n| n.main && n.quarantine == Quarantine::No).nth(0).cloned();
        //if all nodes are in quarantine - take first
        let node = node.unwrap_or_else(|| {
            let n = nodes_.get_mut(&0).expect("There is no nodes defined in config");
            n.main = true;
            n.clone()
        });

        BitcoinClientImpl::new(ctx.client.clone(), node.url.clone(), node.user.clone(), node.password.clone())
    };
    let body = ctx.body.clone();
    Box::new(parse_body::<serde_json::Value>(body).and_then(move |input| {
        let input_clone = input.clone();
        client.proxy_request(&input).map_err(ectx!(ErrorKind::Internal => input_clone))
    }))
}
