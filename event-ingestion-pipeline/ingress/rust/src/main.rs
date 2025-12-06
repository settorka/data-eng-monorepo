mod event;
mod producer;

use axum::{routing::post, Router};
use producer::handler::ingest_event;
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main(){
    tracing_subscriber::fmt::init();

    let api = Router::new()
        .route("/chat/ingestion", post(ingest_event));

    let app = Router::new()
        .nest("/api/v1", api);

    let address = SocketAddr::from(([0,0,0,0],3000));

    tracing::info!("Listening on {}",addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}