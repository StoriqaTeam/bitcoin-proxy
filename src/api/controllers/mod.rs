use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};

use futures::prelude::*;
use hyper::{header::HeaderValue, Body, HeaderMap, Method, Response, Uri};

use super::error::*;
use client::HttpClient;
use config::Config;
use models::*;

mod fallback;
mod proxy;

pub use self::fallback::*;
pub use self::proxy::*;

pub type ControllerFuture = Box<Future<Item = Response<Body>, Error = Error> + Send>;

#[derive(Clone)]
pub struct Context {
    pub body: Vec<u8>,
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap<HeaderValue>,
    pub client: Arc<dyn HttpClient>,
    pub config: Arc<Config>,
    pub nodes: Arc<Mutex<BTreeMap<usize, BitcoinNode>>>,
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!(
            "{} {}, headers: {:#?}, body: {:?}",
            self.method,
            self.uri,
            self.headers,
            String::from_utf8(self.body.clone()).ok()
        ))
    }
}
