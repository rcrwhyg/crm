use anyhow::Result;
use crm::pb::{user_service_client::UserServiceClient, GetUserRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(crm::pb::CreateUserRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    });
    let response = client.create_user(request).await?;
    println!("create_user response: {:?}", response.into_inner());

    let request = Request::new(GetUserRequest { id: 1 });
    let response = client.get_user(request).await?;
    println!("get_user response: {:?}", response.into_inner());

    Ok(())
}
