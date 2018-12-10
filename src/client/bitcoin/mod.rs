mod error;
mod responses;

use std::sync::Arc;

use hyper::{Body, Request};

use self::error::*;
use self::responses::*;
use super::http_client::HttpClient;
use prelude::*;
use serde_json;
use utils::read_body;

/// Client for working with Bitcoin blockchain
pub trait BitcoinClient: Send + Sync + 'static {
    /// Get last block hash
    fn get_last_block(&self) -> Box<Future<Item = u64, Error = Error> + Send>;
}

#[derive(Clone)]
pub struct BitcoinClientImpl {
    http_client: Arc<HttpClient>,
    bitcoin_rpc_url: String,
    bitcoin_rpc_user: String,
    bitcoin_rpc_password: String,
}

impl BitcoinClientImpl {
    pub fn new(http_client: Arc<HttpClient>, bitcoin_rpc_url: String, bitcoin_rpc_user: String, bitcoin_rpc_password: String) -> Self {
        Self {
            http_client,
            bitcoin_rpc_url,
            bitcoin_rpc_user,
            bitcoin_rpc_password,
        }
    }

    fn get_rpc_response<T>(&self, params: &::serde_json::Value) -> impl Future<Item = T, Error = Error> + Send
    where
        for<'a> T: Send + 'static + ::serde::Deserialize<'a>,
    {
        let http_client = self.http_client.clone();
        let params_clone = params.clone();
        let basic = ::base64::encode(&format!("{}:{}", self.bitcoin_rpc_user, self.bitcoin_rpc_password));
        let basic = format!("Basic {}", basic);
        serde_json::to_string(params)
            .map_err(ectx!(ErrorContext::Json, ErrorKind::Internal => params))
            .and_then(|body| {
                Request::builder()
                    .method("POST")
                    .header("Authorization", basic)
                    .uri(self.bitcoin_rpc_url.clone())
                    .body(Body::from(body.clone()))
                    .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => body))
            })
            .into_future()
            .and_then(move |request| http_client.request(request).map_err(ectx!(ErrorKind::Internal)))
            .and_then(|resp| read_body(resp.into_body()).map_err(ectx!(ErrorKind::Internal => params_clone)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ectx!(ErrorContext::UTF8, ErrorKind::Internal => bytes_clone))
            })
            .and_then(|string| serde_json::from_str::<T>(&string).map_err(ectx!(ErrorContext::Json, ErrorKind::Internal => string.clone())))
    }

    fn get_best_block_hash(&self) -> impl Future<Item = String, Error = Error> + Send {
        let params = json!({
            "jsonrpc": "2",
            "id": "1",
            "method": "getbestblockhash",
            "params": []
        });
        self.get_rpc_response::<RpcBestBlockResponse>(&params).map(|r| r.result)
    }

    pub fn get_block_by_hash(&self, hash: String) -> impl Future<Item = Block, Error = Error> + Send {
        let params = json!({
            "jsonrpc": "2",
            "id": "1",
            "method": "getblock",
            "params": [hash]
        });
        self.get_rpc_response::<RpcBlockResponse>(&params).map(|r| r.result)
    }
}

impl BitcoinClient for BitcoinClientImpl {
    fn get_last_block(&self) -> Box<Future<Item = u64, Error = Error> + Send> {
        let self_clone = self.clone();

        Box::new(
            self.get_best_block_hash()
                .and_then(move |hash| self_clone.get_block_by_hash(hash))
                .map(move |block| block.height),
        )
    }
}
