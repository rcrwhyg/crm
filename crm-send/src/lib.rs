use std::{pin::Pin, sync::Arc};

use futures::Stream;
use pb::{notification_server::Notification, send_request::Msg, SendRequest};
use tokio::sync::mpsc;
use tonic::{async_trait, Request, Response, Status, Streaming};

pub use config::AppConfig;

mod abi;
mod config;

pub mod pb;

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<pb::SendResponse, Status>> + Send + 'static>>;

#[derive(Clone)]
pub struct NotificationService {
    inner: Arc<NotificationServiceInner>,
}

#[allow(unused)]
pub struct NotificationServiceInner {
    config: AppConfig,
    sender: mpsc::Sender<Msg>,
}

#[async_trait]
impl Notification for NotificationService {
    type SendStream = ResponseStream;
    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> Result<Response<Self::SendStream>, Status> {
        let stream = request.into_inner();
        self.send(stream).await
    }
}
