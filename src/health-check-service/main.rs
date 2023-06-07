use std::{env, time::Duration};

use tokio::time::sleep;
use uuid::Uuid;

use crate::authentication::{
    auth_client::AuthClient, SignInRequest, SignOutRequest, SignUpRequest, StatusCode,
};
pub mod authentication {
    tonic::include_proto!("authentication");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth_hostname = env::var("AUTH_SERVICE_HOST_NAME").unwrap_or("[::0]".to_owned());
    println!("http://{}:50051", auth_hostname);

    let mut client = AuthClient::connect(format!("http://{}:50051", auth_hostname)).await?;
    loop {
        let username = Uuid::new_v4().to_string();
        let password = Uuid::new_v4().to_string();

        let request = SignUpRequest {
            username: username.clone(),
            password: password.clone(),
        };
        let response = client.sign_up(request).await?.into_inner();
        println!(
            "SIGN UP RESPONSE STATUS: {:?}",
            StatusCode::from_i32(response.status_code)
        );

        // ---------------------------------------------

        let request = SignInRequest { username, password };
        let response = client.sign_in(request).await?.into_inner();
        let session_token = response.session_token;
        println!(
            "SIGN IN RESPONSE STATUS: {:?}",
            StatusCode::from_i32(response.status_code)
        );

        // ---------------------------------------------

        let request = SignOutRequest { session_token };
        let response = client.sign_out(request).await?.into_inner();
        println!(
            "SIGN OUT RESPONSE STATUS: {:?}",
            StatusCode::from_i32(response.status_code)
        );

        sleep(Duration::from_secs(3)).await;
    }
}
