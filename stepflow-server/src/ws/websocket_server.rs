use std::sync::{Arc, Mutex};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::UnboundedReceiver;
use warp::Filter;
use warp::ws::{Message, WebSocket};
use stepflow_hook::ui_event::UiEvent;
use serde_json;

type Clients = Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>>;

/// 启动 WebSocket 服务器，监听 /ws，广播 UiEvent
pub async fn start_ws_server(rx: UnboundedReceiver<UiEvent>) {
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let clients_filter = warp::any().map({
        let clients = clients.clone();
        move || clients.clone()
    });

    // 创建 /ws 路由
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(clients_filter)
        .map(|ws: warp::ws::Ws, clients: Clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients))
        });

    // 启动广播任务
    tokio::spawn(broadcast_ui_events(rx, clients.clone()));

    // 启动服务
    warp::serve(ws_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

/// 处理每个连接的客户端，加入广播池
async fn handle_connection(ws: WebSocket, clients: Clients) {
    let (mut tx, mut rx) = ws.split();
    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // 将 sender 注册到客户端池
    clients.lock().unwrap().push(out_tx);

    // 启动接收客户端 ping/pong 的空 loop
    tokio::spawn(async move {
        while let Some(result) = rx.next().await {
            if result.is_err() {
                break;
            }
        }
    });

    // 推送消息给该客户端
    while let Some(msg) = out_rx.recv().await {
        if tx.send(msg).await.is_err() {
            break;
        }
    }
}

/// 从 UiEvent channel 读取并广播给所有客户端
async fn broadcast_ui_events(mut rx: UnboundedReceiver<UiEvent>, clients: Clients) {
    while let Some(event) = rx.recv().await {
        let payload = match serde_json::to_string(&event) {
            Ok(json) => json,
            Err(_) => continue,
        };

        let msg = Message::text(payload);

        let mut guard = clients.lock().unwrap();
        guard.retain(|client| client.send(msg.clone()).is_ok());
    }
}