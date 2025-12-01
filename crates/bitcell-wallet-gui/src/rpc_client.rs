/// RPC client for BitCell wallet to communicate with the node
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    #[allow(dead_code)]
    id: u64,
}

impl RpcClient {
    pub fn new(host: String, port: u16) -> Self {
        let url = format!("http://{}:{}/rpc", host, port);
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        let json_response: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        if let Some(error) = json_response.error {
            return Err(format!("RPC error: {}", error));
        }

        json_response
            .result
            .ok_or_else(|| "No result in response".to_string())
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<String, String> {
        let params = json!([address, "latest"]);
        let result = self.call("eth_getBalance", params).await?;
        
        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid balance format".to_string())
    }

    /// Send a raw transaction
    pub async fn send_raw_transaction(&self, tx_data: &str) -> Result<String, String> {
        let params = json!([tx_data]);
        let result = self.call("eth_sendRawTransaction", params).await?;
        
        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid transaction hash format".to_string())
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64, String> {
        let params = json!([]);
        let result = self.call("eth_blockNumber", params).await?;
        
        let hex_str = result
            .as_str()
            .ok_or_else(|| "Invalid block number format".to_string())?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| format!("Failed to parse block number: {}", e))
    }

    /// Get node info
    pub async fn get_node_info(&self) -> Result<Value, String> {
        let params = json!([]);
        self.call("bitcell_getNodeInfo", params).await
    }

    /// Get tournament state
    pub async fn get_tournament_state(&self) -> Result<Value, String> {
        let params = json!([]);
        self.call("bitcell_getTournamentState", params).await
    }

    /// Get battle replay
    pub async fn get_battle_replay(&self, block_height: u64) -> Result<Value, String> {
        let params = json!([block_height]);
        self.call("bitcell_getBattleReplay", params).await
    }
}
