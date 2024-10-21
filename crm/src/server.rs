use anyhow::Result;
use crm::pb::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserRequest, GetUserRequest, User,
};
use tonic::transport::Server;

#[derive(Default)]
pub struct UserServer {}

#[tonic::async_trait]
impl UserService for UserServer {
    async fn create_user(
        &self,
        request: tonic::Request<CreateUserRequest>,
    ) -> Result<tonic::Response<User>, tonic::Status> {
        let input = request.into_inner();
        println!("create_user: {:?}", input);
        let user = User::new(1, &input.name, &input.email);
        Ok(tonic::Response::new(user))
    }

    async fn get_user(
        &self,
        request: tonic::Request<GetUserRequest>,
    ) -> Result<tonic::Response<User>, tonic::Status> {
        let input = request.into_inner();
        println!("get_user: {:?}", input);
        Ok(tonic::Response::new(User::default()))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse().unwrap();
    let svc = UserServer {};

    println!("UserService listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
