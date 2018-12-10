#[derive(Debug, Clone, Deserialize)]
pub struct RpcBlockResponse {
    pub result: Block,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Block {
    pub hash: String,
    pub previousblockhash: String,
    pub tx: Vec<String>,
    pub height: u64,
    pub confirmations: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcBestBlockResponse {
    pub result: String,
}
