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

    /// Get transaction count (nonce) for an address
    pub async fn get_transaction_count(&self, address: &str) -> Result<u64, String> {
        let params = json!([address, "latest"]);
        let result = self.call("eth_getTransactionCount", params).await?;
        
        let hex_str = result
            .as_str()
            .ok_or_else(|| "Invalid nonce format".to_string())?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| format!("Failed to parse nonce: {}", e))
    }

    /// Send a raw transaction (hex-encoded signed transaction)
    pub async fn send_raw_transaction(&self, tx_hex: &str) -> Result<String, String> {
        let params = json!([tx_hex]);
        let result = self.call("eth_sendRawTransaction", params).await?;
        
        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid transaction hash format".to_string())
    }

    /// Send a raw transaction (bytes)
    pub async fn send_raw_transaction_bytes(&self, tx_bytes: &[u8]) -> Result<String, String> {
        let tx_hex = format!("0x{}", hex::encode(tx_bytes));
        self.send_raw_transaction(&tx_hex).await
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

    /// Get gas price
    pub async fn get_gas_price(&self) -> Result<u64, String> {
        let params = json!([]);
        let result = self.call("eth_gasPrice", params).await?;
        
        let hex_str = result
            .as_str()
            .ok_or_else(|| "Invalid gas price format".to_string())?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| format!("Failed to parse gas price: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_client_construction() {
        let client = RpcClient::new("127.0.0.1".to_string(), 30334);
        assert_eq!(client.url, "http://127.0.0.1:30334/rpc");
    }

    #[test]
    fn test_rpc_client_url_format() {
        let client = RpcClient::new("localhost".to_string(), 8545);
        assert_eq!(client.url, "http://localhost:8545/rpc");
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: json!([]),
            id: 1,
        };
        
        let json_str = serde_json::to_string(&request).unwrap();
        assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        assert!(json_str.contains("\"method\":\"eth_blockNumber\""));
        assert!(json_str.contains("\"id\":1"));
    }

    #[test]
    fn test_json_rpc_response_deserialization() {
        let json_str = r#"{
            "jsonrpc": "2.0",
            "result": "0x10",
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert_eq!(response.result.unwrap(), json!("0x10"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_error_response_deserialization() {
        let json_str = r#"{
            "jsonrpc": "2.0",
            "error": {"code": -32602, "message": "Invalid params"},
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error["code"], -32602);
    }

    #[test]
    fn test_block_number_hex_parsing() {
        // Test parsing various hex formats
        let hex1 = "0x10";
        let parsed1 = u64::from_str_radix(hex1.trim_start_matches("0x"), 16);
        assert_eq!(parsed1.unwrap(), 16);
        
        let hex2 = "0xff";
        let parsed2 = u64::from_str_radix(hex2.trim_start_matches("0x"), 16);
        assert_eq!(parsed2.unwrap(), 255);
        
        let hex3 = "0x3039"; // 12345
        let parsed3 = u64::from_str_radix(hex3.trim_start_matches("0x"), 16);
        assert_eq!(parsed3.unwrap(), 12345);
    }
}
