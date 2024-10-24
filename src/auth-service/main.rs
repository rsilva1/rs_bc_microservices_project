mod auth;
mod users;
mod sessions;

use std::sync::Mutex;

use auth::{authentication::auth_server::AuthServer, AuthService};
use sessions::SessionsImpl;
use tonic::transport::Server;
use users::UsersImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::0]:50051".parse()?;

    let users_service = Box::new(Mutex::new(UsersImpl::default()));
    let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

    let auth_service = AuthService::new(users_service, sessions_service);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
