mod error;

use std::sync::Arc;

use failure::Fail;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use serde_json;

pub use self::error::*;
use super::HttpClient;
use config::Config;
use utils::read_body;

pub trait BlockchainInfoClient: Send + Sync + 'static {
    fn get_block_count(&self) -> Box<Future<Item = u64, Error = Error> + Send>;
}

#[derive(Clone)]
pub struct BlockchainInfoClientImpl {
    cli: Arc<HttpClient>,
    url: String,
}

impl BlockchainInfoClientImpl {
    pub fn new<C: HttpClient>(config: &Config, cli: C) -> Self {
        Self {
            cli: Arc::new(cli),
            url: config.healthcheck.url.clone(),
        }
    }

    fn exec_query(&self) -> impl Future<Item = u64, Error = Error> + Send {
        let url = self.url.clone();
        let query = url.clone();
        let query1 = query.clone();
        let query2 = query.clone();
        let cli = self.cli.clone();
        let mut builder = Request::builder();
        builder.uri(url).method(Method::GET);
        builder.header("user-agent", "Mozilla/5.0 (X11; Ubuntu; Linu…) Gecko/20100101 Firefox/63.0");
        builder
            .body(Body::empty())
            .map_err(ectx!(ErrorSource::Hyper, ErrorKind::MalformedInput))
            .into_future()
            .and_then(move |req| cli.request(req).map_err(ectx!(ErrorKind::Internal => query1)))
            .and_then(move |resp| read_body(resp.into_body()).map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => query2)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ectx!(ErrorSource::Utf8, ErrorKind::Internal => bytes_clone))
            })
            .and_then(|string| serde_json::from_str::<u64>(&string).map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => string)))
    }
}

impl BlockchainInfoClient for BlockchainInfoClientImpl {
    fn get_block_count(&self) -> Box<Future<Item = u64, Error = Error> + Send> {
        Box::new(self.exec_query())
    }
}

#[derive(Default)]
pub struct BlockchainInfoClientMock;

impl BlockchainInfoClient for BlockchainInfoClientMock {
    fn get_block_count(&self) -> Box<Future<Item = u64, Error = Error> + Send> {
        Box::new(Ok(55563).into_future())
    }
}
