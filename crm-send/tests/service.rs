use anyhow::Result;
use crm_send::pb::notification_client::NotificationClient;
use crm_send::pb::{EmailMessage, InAppMessage, SendRequest, SmsMessage};
use crm_send::{AppConfig, NotificationService};
use futures::StreamExt;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::Request;

#[tokio::test]
async fn test_send() -> Result<()> {
    let addr = start_server().await?;
    let mut client = NotificationClient::connect(format!("http://{addr}")).await?;
    let stream = tokio_stream::iter(vec![
        SendRequest {
            msg: Some(EmailMessage::fake().into()),
        },
        SendRequest {
            msg: Some(SmsMessage::fake().into()),
        },
        SendRequest {
            msg: Some(InAppMessage::fake().into()),
        },
    ]);
    let request = Request::new(stream);
    let response = client.send(request).await?.into_inner();
    let ret: Vec<_> = response
        .then(|resp| async { resp.unwrap() })
        .collect()
        .await;

    assert_eq!(ret.len(), 3);

    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::try_load()?;
    let addr = format!("[::1]:{}", config.server.port).parse()?;

    let svc = NotificationService::new(config).into_server();

    tokio::spawn(async move {
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .expect("Failed to start server");
    });

    Ok(addr)
}
