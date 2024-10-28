use anyhow::Result;
use crm::pb::crm_client::CrmClient;
use crm::pb::WelcomeRequestBuilder;
use tonic::metadata::MetadataValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::Request;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let pem = include_str!("../../fixtures/rootCA.pem");
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(pem))
        .domain_name("localhost");
    let channel = Channel::from_static("https://[::1]:50000")
        .tls_config(tls)?
        .connect()
        .await?;

    let token = include_str!("../../fixtures/token").trim();
    let token: MetadataValue<_> = format!("Bearer {}", token).parse()?;

    let mut client = CrmClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    let request = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(93u32)
        .content_ids([1u32, 2, 3])
        .build()?;

    let response = client.welcome(Request::new(request)).await?.into_inner();
    println!("Response: {:?}", response);

    Ok(())
}
