mod ws;

use stepflow_hook::ui_event::UiEvent;

#[tokio::main]
async fn main() {
    // 启动 WebSocket 广播服务
    let (_tx, rx) = tokio::sync::mpsc::unbounded_channel::<UiEvent>();
    tokio::spawn(ws::websocket_server::start_ws_server(rx));

    println!("✅ WebSocket server running at ws://localhost:3030/ws");

    // 阻塞主线程（本例中暂不提供其他路由）
    tokio::signal::ctrl_c().await.unwrap();
}