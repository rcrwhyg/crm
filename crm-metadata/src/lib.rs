use std::pin::Pin;

use futures::Stream;
use pb::{
    metadata_server::{Metadata, MetadataServer},
    Content, MaterializeRequest,
};
use tonic::{async_trait, Request, Response, Status, Streaming};

pub use abi::Tpl;
pub use config::AppConfig;

mod abi;
mod config;

pub mod pb;

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;

#[allow(unused)]
pub struct MetadataService {
    config: AppConfig,
}

#[async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;

    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> Result<Response<Self::MaterializeStream>, Status> {
        let query = request.into_inner();
        self.materialize(query).await
    }
}

impl MetadataService {
    pub async fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}
