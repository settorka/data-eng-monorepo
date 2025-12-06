use tonic::transport::Channel;
use tonic::Request;
use crate::event::Event;

pub mod chat {
    tonic::include_proto!("chat"); 
}

use chat::{router_client::RouterClient, IngestResponse};

pub struct ElixirGrpcClient {
    client: RouterClient<Channel>,
}

impl ElixirGrpcClient {
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let client = RouterClient::connect(dst).await?;
        Ok(Self { client })
    }

    pub async fn ingest_event(&mut self, event: chat::Event) -> Result<IngestResponse, tonic::Status> {
        let req = Request::new(event);
        let res = self.client.ingest(req).await?;
        Ok(res.into_inner())
    }
}
