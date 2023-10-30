pub mod config;
pub mod errors;

use std::net::SocketAddr;

use axum::Router;
use listenfd::ListenFd;
use tokio::net::TcpListener;

pub async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        Some(listener) => TcpListener::from_std(listener).unwrap(),
        None => TcpListener::bind(addr).await.unwrap(),
    };

    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
