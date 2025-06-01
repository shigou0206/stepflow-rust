use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use tokio::sync::broadcast;
use crate::app_state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    state: axum::extract::State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: axum::extract::State<AppState>) {
    let (tx, mut rx) = broadcast::channel(100);

    // 发送初始状态
    if let Err(e) = socket.send(Message::Text("Connected to StepFlow WebSocket".into())).await {
        tracing::error!("Failed to send welcome message: {}", e);
        return;
    }

    loop {
        tokio::select! {
            // 处理从客户端接收的消息
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Text(text) => {
                        tracing::debug!("Received message: {}", text);
                        // 这里可以处理接收到的消息
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            }
            // 处理需要发送给客户端的消息
            Ok(msg) = rx.recv() => {
                if let Err(e) = socket.send(Message::Text(msg)).await {
                    tracing::error!("Failed to send message: {}", e);
                    break;
                }
            }
            else => break
        }
    }
} 