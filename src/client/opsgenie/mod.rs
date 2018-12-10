mod error;
mod responses;

use std::sync::Arc;

use failure::Fail;
use futures::future;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use serde::Deserialize;
use serde_json;

pub use self::error::*;
use self::responses::*;
use super::HttpClient;
use config::Config;
use utils::read_body;

pub trait OpsGenieClient: Send + Sync + 'static {
    fn notify(&self, message: String) -> Box<Future<Item = (), Error = Error> + Send>;
}

#[derive(Clone)]
pub struct OpsGenieClientImpl {
    cli: Arc<HttpClient>,
    enabled: bool,
    api_key: String,
    url: String,
    team: String,
}

impl OpsGenieClientImpl {
    pub fn new<C: HttpClient>(config: &Config, cli: C) -> Self {
        Self {
            cli: Arc::new(cli),
            enabled: config.opsgenie.enabled,
            api_key: config.opsgenie.api_key.clone(),
            url: config.opsgenie.url.clone(),
            team: config.opsgenie.team.clone(),
        }
    }

    fn exec_query<T: for<'de> Deserialize<'de> + Send>(&self, body: String) -> impl Future<Item = T, Error = Error> + Send {
        let url = self.url.clone();
        let query1 = url.clone();
        let query2 = url.clone();
        let query3 = url.clone();
        let cli = self.cli.clone();
        let mut builder = Request::builder();
        let token = self.api_key.clone();
        builder.uri(url).method(Method::POST);
        builder.header("Authorization", format!("GenieKey {}", token));
        builder
            .body(Body::from(body))
            .map_err(ectx!(ErrorSource::Hyper, ErrorKind::MalformedInput => query3))
            .into_future()
            .and_then(move |req| cli.request(req).map_err(ectx!(ErrorKind::Internal => query1)))
            .and_then(move |resp| read_body(resp.into_body()).map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => query2)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ectx!(ErrorSource::Utf8, ErrorKind::Internal => bytes_clone))
            })
            .and_then(|string| serde_json::from_str::<T>(&string).map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => string)))
    }
}

impl OpsGenieClient for OpsGenieClientImpl {
    fn notify(&self, message: String) -> Box<Future<Item = (), Error = Error> + Send> {
        if self.enabled {
            let client = self.clone();
            let team = self.team.clone();
            let payload = OpsGeniePayload::new(message, team);
            Box::new(
                serde_json::to_string(&payload)
                    .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => payload))
                    .into_future()
                    .and_then(move |body| client.exec_query::<OpsGenieResponse>(body).map(move |_| ())),
            )
        } else {
            Box::new(future::ok(()))
        }
    }
}

#[derive(Default)]
pub struct OpsGenieClientMock;

impl OpsGenieClient for OpsGenieClientMock {
    fn notify(&self, _message: String) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(Ok(()).into_future())
    }
}
