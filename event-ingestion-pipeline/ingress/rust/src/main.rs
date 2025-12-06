mod event;
mod forwarder;

use axum::{Router, routing::post};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber;

use forwarder::grpc::handler::ingest_event;
use forwarder::grpc::elixir_client::ElixirGrpcClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // gRPC backend setup
    let grpc_client = ElixirGrpcClient::connect("http://[::1]:50051")
        .await
        .expect("Failed to connect to Elixir router");

    let grpc_client = Arc::new(Mutex::new(grpc_client));

    // API router config
    let api = Router::new()
        .route("/chat/ingestion", post(ingest_event))
        .with_state(grpc_client);

    let app = Router::new()
        .nest("/api/v1", api);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);
    
    // exec!
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
