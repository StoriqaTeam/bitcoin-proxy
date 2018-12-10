use std::collections::BTreeMap;
use std::env;

use config_crate::{Config as RawConfig, ConfigError, Environment, File};
use logger::{FileLogConfig, GrayLogConfig};
use models::*;
use sentry_integration::SentryConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub client: Client,
    pub cpu_pool: CpuPool,
    pub nodes: Vec<Node>,
    pub healthcheck: Healthcheck,
    pub opsgenie: OpsGenie,
    pub sentry: Option<SentryConfig>,
    pub graylog: Option<GrayLogConfig>,
    pub filelog: Option<FileLogConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Healthcheck {
    pub timeout: u64,
    pub url: String,
    pub quarantine: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub dns_threads: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_user: String,
    pub bitcoin_rpc_password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpsGenie {
    pub enabled: bool,
    pub api_key: String,
    pub url: String,
    pub team: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CpuPool {
    pub size: usize,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Merge development.toml if RUN_MODE variable is not set
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;
        s.merge(File::with_name("config/secret.toml").required(false))?;

        s.merge(Environment::with_prefix("STQ_PAYMENTS"))?;
        s.try_into()
    }

    pub fn to_nodes(&self) -> BTreeMap<usize, BitcoinNode> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                (
                    i,
                    BitcoinNode::new(
                        node.bitcoin_rpc_url.clone(),
                        node.bitcoin_rpc_user.clone(),
                        node.bitcoin_rpc_password.clone(),
                    ),
                )
            })
            .collect()
    }
}
