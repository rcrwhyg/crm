use std::mem;

use anyhow::Result;
use crm::{AppConfig, CrmService};
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let mut config = AppConfig::try_load().expect("Failed to load config");

    let tls = mem::take(&mut config.server.tls);

    let port = &config.server.port;
    let addr = format!("[::1]:{port}").parse()?;
    info!("CRM service listening on {}", addr);
    let svc = CrmService::try_new(config).await?.into_server()?;

    if let Some(tls) = tls {
        let identity = Identity::from_pem(tls.cert, tls.key);
        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(svc)
            .serve(addr)
            .await?;
    } else {
        Server::builder().add_service(svc).serve(addr).await?;
    }

    Ok(())
}
