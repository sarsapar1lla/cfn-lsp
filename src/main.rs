#![allow(dead_code)]

use axum::{body::Bytes, extract::State, response::IntoResponse, routing::get, Json, Router};
use handler::Server;
use model::Message;
use tokio::net::TcpListener;
use tracing_subscriber::util::SubscriberInitExt;

mod handler;
mod method;
mod model;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::FmtSubscriber::new().init();
    let server = Server::default();
    let app = Router::new().route("/", get(handle)).with_state(server);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle(State(server): State<Server>, body: Bytes) -> impl IntoResponse {
    let message: Message = serde_json::from_slice(&body).unwrap();
    let response = server.handle(message);
    Json(response)
}
