use anyhow::Result;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_metadata::pb::MaterializeRequest;
use crm_metadata::{AppConfig, MetadataService};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tonic::transport::Server;
use tonic::Request;

#[tokio::test]
async fn test_metadata() -> Result<()> {
    let addr = start_server().await?;
    let mut client = MetadataClient::connect(format!("http://{addr}")).await?;
    let stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
        MaterializeRequest { id: 3 },
    ]);
    let request = Request::new(stream);
    let response = client.materialize(request).await?.into_inner();
    let ret: Vec<_> = response
        .then(|item| async move { item.unwrap() })
        .collect()
        .await;

    assert_eq!(ret.len(), 3);

    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::try_load()?;
    let addr = format!("[::1]:{}", config.server.port).parse()?;

    let svc = MetadataService::new(config).await.into_server();

    tokio::spawn(async move {
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .expect("Failed to start server");
    });

    sleep(Duration::from_micros(1)).await;

    Ok(addr)
}
