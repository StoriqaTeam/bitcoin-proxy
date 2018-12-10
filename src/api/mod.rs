mod controllers;
mod error;
mod utils;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use failure::{Compat, Fail};
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper;
use hyper::Server;
use hyper::{service::Service, Body, Request, Response};

use self::controllers::*;
use self::error::*;
use super::config::Config;
use super::utils::{log_and_capture_error, log_error, log_warn};
use client::{HttpClient, HttpClientImpl};
use models::*;
use utils::read_body;

#[derive(Clone)]
pub struct ApiService {
    server_address: SocketAddr,
    config: Arc<Config>,
    cpu_pool: CpuPool,
    client: Arc<dyn HttpClient>,
    nodes: Arc<Mutex<BTreeMap<usize, BitcoinNode>>>,
}

impl ApiService {
    fn from_config(config: Config, nodes: Arc<Mutex<BTreeMap<usize, BitcoinNode>>>) -> Result<Self, Error> {
        let client = HttpClientImpl::new(&config);
        let host = config.server.host.clone();
        let port = config.server.port.clone();
        let server_address = format!("{}:{}", host, port).parse::<SocketAddr>().map_err(ectx!(try
            ErrorContext::Config,
            ErrorKind::Internal =>
            host,
            port
        ))?;
        let cpu_pool = CpuPool::new(config.cpu_pool.size);
        Ok(ApiService {
            config: Arc::new(config),
            server_address,
            cpu_pool,
            client: Arc::new(client),
            nodes,
        })
    }
}

impl Service for ApiService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, http_body) = req.into_parts();
        let client = self.client.clone();
        let config = self.config.clone();
        let nodes = self.nodes.clone();

        Box::new(
            read_body(http_body)
                .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal))
                .and_then(move |body| {
                    let ctx = Context {
                        body,
                        method: parts.method.clone(),
                        uri: parts.uri.clone(),
                        headers: parts.headers,
                        client,
                        config,
                        nodes,
                    };

                    debug!("Received request {}", ctx);

                    proxy(&ctx)
                })
                .and_then(|resp| {
                    let (parts, body) = resp.into_parts();
                    read_body(body)
                        .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal))
                        .map(|body| (parts, body))
                })
                .map(|(parts, body)| {
                    debug!(
                        "Sent response with status {}, headers: {:#?}, body: {:?}",
                        parts.status.as_u16(),
                        parts.headers,
                        String::from_utf8(body.clone()).ok()
                    );
                    Response::from_parts(parts, body.into())
                })
                .or_else(|e| match e.kind() {
                    ErrorKind::BadRequest => {
                        log_error(&e);
                        Ok(Response::builder()
                            .status(400)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Bad request"}"#))
                            .unwrap())
                    }
                    ErrorKind::Unauthorized => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(401)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Unauthorized"}"#))
                            .unwrap())
                    }
                    ErrorKind::NotFound => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(404)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Not found"}"#))
                            .unwrap())
                    }
                    ErrorKind::UnprocessableEntity(errors) => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(422)
                            .header("Content-Type", "application/json")
                            .body(Body::from(errors))
                            .unwrap())
                    }
                    ErrorKind::Internal => {
                        log_and_capture_error(e);
                        Ok(Response::builder()
                            .status(500)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Internal server error"}"#))
                            .unwrap())
                    }
                }),
        )
    }
}

pub fn start_server(config: Config, nodes: Arc<Mutex<BTreeMap<usize, BitcoinNode>>>) {
    hyper::rt::run(future::lazy(move || {
        ApiService::from_config(config, nodes)
            .into_future()
            .and_then(move |api| {
                let api_clone = api.clone();
                let new_service = move || {
                    let res: Result<_, hyper::Error> = Ok(api_clone.clone());
                    res
                };
                let addr = api.server_address.clone();
                let server = Server::bind(&api.server_address)
                    .serve(new_service)
                    .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => addr));
                info!("Listening on http://{}", addr);
                server
            })
            .map_err(|e: Error| log_error(&e))
    }));
}
