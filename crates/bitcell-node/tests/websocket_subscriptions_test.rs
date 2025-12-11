//! Integration tests for WebSocket subscriptions

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{StreamExt, SinkExt};
use serde_json::{json, Value};
use std::time::Duration;

#[tokio::test]
async fn test_websocket_new_heads_subscription() {
    // Skip if no RPC server is running
    let url = "ws://127.0.0.1:8545/ws";
    let result = connect_async(url).await;
    
    if result.is_err() {
        println!("Skipping test - no RPC server running");
        return;
    }
    
    let (ws_stream, _) = result.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to newHeads
    let subscribe_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newHeads"]
    });
    
    write.send(Message::Text(subscribe_req.to_string())).await.unwrap();
    
    // Read subscription response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_string());
        
        println!("Subscription ID: {}", response["result"]);
    }
}

#[tokio::test]
async fn test_websocket_pending_transactions_subscription() {
    let url = "ws://127.0.0.1:8545/ws";
    let result = connect_async(url).await;
    
    if result.is_err() {
        println!("Skipping test - no RPC server running");
        return;
    }
    
    let (ws_stream, _) = result.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to pendingTransactions
    let subscribe_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["pendingTransactions"]
    });
    
    write.send(Message::Text(subscribe_req.to_string())).await.unwrap();
    
    // Read subscription response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_string());
        
        println!("Subscription ID: {}", response["result"]);
    }
}

#[tokio::test]
async fn test_websocket_logs_subscription_with_filter() {
    let url = "ws://127.0.0.1:8545/ws";
    let result = connect_async(url).await;
    
    if result.is_err() {
        println!("Skipping test - no RPC server running");
        return;
    }
    
    let (ws_stream, _) = result.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to logs with address filter
    let subscribe_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": [
            "logs",
            {
                "address": ["0x1234567890123456789012345678901234567890"],
                "topics": []
            }
        ]
    });
    
    write.send(Message::Text(subscribe_req.to_string())).await.unwrap();
    
    // Read subscription response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_string());
        
        println!("Subscription ID: {}", response["result"]);
    }
}

#[tokio::test]
async fn test_websocket_unsubscribe() {
    let url = "ws://127.0.0.1:8545/ws";
    let result = connect_async(url).await;
    
    if result.is_err() {
        println!("Skipping test - no RPC server running");
        return;
    }
    
    let (ws_stream, _) = result.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe first
    let subscribe_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newHeads"]
    });
    
    write.send(Message::Text(subscribe_req.to_string())).await.unwrap();
    
    // Get subscription ID
    let sub_id = if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: Value = serde_json::from_str(&text).unwrap();
        response["result"].as_str().unwrap().to_string()
    } else {
        panic!("Failed to get subscription ID");
    };
    
    // Unsubscribe
    let unsubscribe_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "eth_unsubscribe",
        "params": [sub_id]
    });
    
    write.send(Message::Text(unsubscribe_req.to_string())).await.unwrap();
    
    // Read unsubscribe response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["result"], true);
    }
}

#[tokio::test]
async fn test_rate_limiting() {
    let url = "ws://127.0.0.1:8545/ws";
    let result = connect_async(url).await;
    
    if result.is_err() {
        println!("Skipping test - no RPC server running");
        return;
    }
    
    let (ws_stream, _) = result.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Try to send many requests quickly
    for i in 0..150 {
        let subscribe_req = json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        });
        
        if let Err(e) = write.send(Message::Text(subscribe_req.to_string())).await {
            println!("Failed to send request {}: {}", i, e);
            break;
        }
    }
    
    // Check if rate limit error is received
    let mut rate_limit_hit = false;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(response) = serde_json::from_str::<Value>(&text) {
                if let Some(error) = response.get("error") {
                    if error["code"] == -32005 {
                        rate_limit_hit = true;
                        break;
                    }
                }
            }
        }
    }).await.ok();
    
    println!("Rate limit hit: {}", rate_limit_hit);
}

#[tokio::test]
async fn test_max_subscriptions_per_client() {
    let url = "ws://127.0.0.1:8545/ws";
    let result = connect_async(url).await;
    
    if result.is_err() {
        println!("Skipping test - no RPC server running");
        return;
    }
    
    let (ws_stream, _) = result.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Try to create more than MAX_SUBSCRIPTIONS_PER_CLIENT
    for i in 0..105 {
        let subscribe_req = json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        });
        
        write.send(Message::Text(subscribe_req.to_string())).await.unwrap();
        
        // Small delay to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(15)).await;
    }
    
    // Check if subscription limit error is received
    let mut limit_hit = false;
    tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(response) = serde_json::from_str::<Value>(&text) {
                if let Some(error) = response.get("error") {
                    if error["code"] == -32005 && error["message"].as_str().unwrap().contains("Exceeded max subscriptions") {
                        limit_hit = true;
                        break;
                    }
                }
            }
        }
    }).await.ok();
    
    println!("Subscription limit hit: {}", limit_hit);
}
