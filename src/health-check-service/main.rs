use std::{env, error::Error, time::Duration};

use authentication::{auth_client::AuthClient, SignInRequest, SignOutRequest, SignUpRequest, StatusCode};
use tokio::time::sleep;
use tonic::Request;
use uuid::Uuid;

pub mod authentication {
    tonic::include_proto!("authentication");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let auth_hostname = env::var("AUTH_SERVICE_HOST_NAME").unwrap_or("[::0]".to_owned());

    let mut client = AuthClient::connect(format!("http://{}:50051", auth_hostname)).await?;

    loop {
        let username = Uuid::new_v4().to_string();
        let password = Uuid::new_v4().to_string();

        let request = Request::new(SignUpRequest {
            username: username.clone(),
            password: password.clone(),
        });

        let response = client.sign_up(request).await?;

        println!(
            "SIGN UP RESPONSE STATUS: {:?}",
            StatusCode::from_i32(response.into_inner().status_code)
        );

        // --------------------------------------

        let request = Request::new(SignInRequest {
            username: username.clone(),
            password: password.clone(),
        });

        let response = client.sign_in(request).await?;
        let sign_in_response = response.into_inner();

        println!(
            "SIGN IN RESPONSE STATUS: {:?}",
            StatusCode::from_i32(sign_in_response.status_code)
        );

        // --------------------------------------
        
        let session_token = sign_in_response.session_token;
        let request = Request::new(SignOutRequest {
            session_token
        });

        let response = client.sign_out(request).await?;

        println!(
            "SIGN OUT RESPONSE STATUS: {:?}",
            StatusCode::from_i32(response.into_inner().status_code)
        );

        println!("------------------------------");

        sleep(Duration::from_secs(3)).await;
    }
}
