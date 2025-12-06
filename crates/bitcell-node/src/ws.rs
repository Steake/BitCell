use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::time::Duration;
use tokio::time;
use crate::rpc::RpcState;
use serde_json::json;

pub fn ws_router() -> Router<RpcState> {
    Router::new()
        .route("/battles", get(battles_handler))
        .route("/blocks", get(blocks_handler))
}

async fn battles_handler(
    ws: WebSocketUpgrade,
    State(state): State<RpcState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_battles_socket(socket, state))
}

async fn blocks_handler(
    ws: WebSocketUpgrade,
    State(state): State<RpcState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_blocks_socket(socket, state))
}

async fn handle_battles_socket(mut socket: WebSocket, state: RpcState) {
    let mut interval = time::interval(Duration::from_secs(1));
    let mut last_phase = "unknown".to_string();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Some(tm) = &state.tournament_manager {
                    let current_phase = match tm.current_phase().await {
                        Some(p) => format!("{:?}", p).to_lowercase(),
                        None => "idle".to_string(),
                    };

                    if current_phase != last_phase {
                        let msg = json!({
                            "type": "phase_change",
                            "phase": current_phase,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });

                        if let Err(e) = socket.send(Message::Text(msg.to_string())).await {
                            tracing::error!("Failed to send battle update: {}", e);
                            break;
                        }
                        last_phase = current_phase;
                    }
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Close(_))) = msg {
                    break;
                }
                if let None = msg {
                    break;
                }
            }
        }
    }
}

async fn handle_blocks_socket(mut socket: WebSocket, state: RpcState) {
    let mut interval = time::interval(Duration::from_secs(5));
    let mut last_height = state.blockchain.height();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let current_height = state.blockchain.height();
                if current_height > last_height {
                    let msg = json!({
                        "type": "new_block",
                        "height": current_height,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    if let Err(e) = socket.send(Message::Text(msg.to_string())).await {
                        tracing::error!("Failed to send block update: {}", e);
                        break;
                    }
                    last_height = current_height;
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Close(_))) = msg {
                    break;
                }
                if let None = msg {
                    break;
                }
            }
        }
    }
}
