use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use crm_metadata::pb::Content;
use crm_metadata::Tpl;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt as _};
use tonic::{Response, Status};
use tracing::{info, warn};
use uuid::Uuid;

use crate::pb::send_request::Msg;
use crate::pb::{EmailMessage, SendResponse};
use crate::NotificationServiceInner;
use crate::{
    pb::{notification_server::NotificationServer, SendRequest},
    AppConfig, NotificationService, ResponseStream, ServiceResult,
};

mod email;
mod in_app;
mod sms;

const CHANNEL_SIZE: usize = 1024;

pub trait Sender {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status>;
}

impl NotificationService {
    pub fn new(config: AppConfig) -> Self {
        let sender = dummy_send();
        let inner = NotificationServiceInner { config, sender };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }

    pub async fn send(
        &self,
        mut stream: impl Stream<Item = Result<SendRequest, Status>> + Send + 'static + Unpin,
    ) -> ServiceResult<ResponseStream> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        let notify = self.clone();

        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                let notify_clone = notify.clone();
                let resp = match req.msg {
                    Some(Msg::Email(email)) => email.send(notify_clone).await,
                    Some(Msg::Sms(sms)) => sms.send(notify_clone).await,
                    Some(Msg::InApp(in_app)) => in_app.send(notify_clone).await,
                    None => {
                        warn!("Invalid message type");
                        Err(Status::invalid_argument("Invalid message"))
                    }
                };
                tx.send(resp).await.unwrap();
            }
        });

        let stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(stream)))
    }
}

impl Deref for NotificationService {
    type Target = NotificationServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SendRequest {
    pub fn new(
        subject: String,
        sender: String,
        recipients: &[String],
        contents: &[Content],
    ) -> Self {
        let tpl = Tpl(contents);
        let msg = Msg::Email(EmailMessage {
            message_id: Uuid::new_v4().to_string(),
            subject,
            sender,
            recipients: recipients.to_vec(),
            body: tpl.to_body(),
        });

        SendRequest { msg: Some(msg) }
    }
}

fn dummy_send() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(CHANNEL_SIZE * 100);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Sending message: {:?}", msg);
            sleep(Duration::from_millis(300)).await;
        }
    });
    tx
}

fn to_ts() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

#[cfg(test)]
mod tests {
    // use crate::pb::{InAppMessage, SmsMessage};

    // use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_send_should_work() -> Result<()> {
        // let config = AppConfig::try_load()?;
        // let service = NotificationService::new(config);
        // let stream = tokio_stream::iter(vec![
        //     Ok(EmailMessage::fake().into()),
        //     Ok(SmsMessage::fake().into()),
        //     Ok(InAppMessage::fake().into()),
        // ]);

        // let response = service.send(stream).await?;
        // let ret = response.into_inner().collect::<Vec<_>>().await;
        // assert_eq!(ret.len(), 3);

        Ok(())
    }
}
