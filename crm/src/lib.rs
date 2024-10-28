mod abi;
mod config;

pub mod pb;

pub use config::AppConfig;

use crate::abi::auth;
use crate::pb::crm_server::{Crm, CrmServer};
use crate::pb::{
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use anyhow::Result;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;
use tonic::{async_trait, Request, Response, Status};
use tracing::info;
use user_stat::pb::user_stats_client::UserStatsClient;

pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> std::result::Result<Response<WelcomeResponse>, Status> {
        let user: &auth::User = request.extensions().get().expect("");
        info!("User: {:?}", user);
        self.welcome(request.into_inner()).await
    }

    async fn recall(
        &self,
        _request: Request<RecallRequest>,
    ) -> std::result::Result<Response<RecallResponse>, Status> {
        todo!()
    }

    async fn remind(
        &self,
        _request: Request<RemindRequest>,
    ) -> std::result::Result<Response<RemindResponse>, Status> {
        todo!()
    }
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        let user_stats = UserStatsClient::connect(config.server.user_stats.clone()).await?;
        let notification = NotificationClient::connect(config.server.notification.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata.clone()).await?;

        Ok(Self {
            config,
            user_stats,
            notification,
            metadata,
        })
    }

    pub fn into_server(self) -> Result<InterceptedService<CrmServer<Self>, auth::DecodingKey>> {
        let dk = auth::DecodingKey::load(&self.config.auth.pk)?;
        Ok(CrmServer::with_interceptor(self, dk))
    }
}
