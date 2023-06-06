use std::sync::Mutex;

use crate::{sessions::Sessions, users::Users};

use tonic::{Request, Response, Status};

use authentication::auth_server::Auth;
use authentication::{
    SignInRequest, SignInResponse, SignOutRequest, SignOutResponse, SignUpRequest, SignUpResponse,
    StatusCode,
};

pub mod authentication {
    tonic::include_proto!("authentication");
}

pub use authentication::auth_server::AuthServer;
pub use tonic::transport::Server;

pub struct AuthService {
    users_service: Box<Mutex<dyn Users + Send + Sync>>,
    sessions_service: Box<Mutex<dyn Sessions + Send + Sync>>,
}

impl AuthService {
    pub fn new(
        users_service: Box<Mutex<dyn Users + Send + Sync>>,
        sessions_service: Box<Mutex<dyn Sessions + Send + Sync>>,
    ) -> Self {
        Self {
            users_service,
            sessions_service,
        }
    }
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn sign_in(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        println!("Got a request: {:?}", request);
        let req = request.into_inner();
        let result: Option<String> = self
            .users_service
            .lock()
            .expect("Failed to get users_service on auth")
            .get_user_uuid(req.username, req.password);

        let Some(user_uuid) = result else {
            let response = SignInResponse {
                status_code: StatusCode::Failure.into(),
                user_uuid: "".to_owned(),
                session_token: "".to_owned(),
            };
            return Ok(Response::new(response));
        };

        let session_token = self
            .sessions_service
            .lock()
            .expect("Failed to get session_service on auth")
            .create_session(&user_uuid);

        let reply = SignInResponse {
            status_code: StatusCode::Success.into(),
            session_token,
            user_uuid,
        };
        Ok(Response::new(reply))
    }

    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        let req = request.into_inner();
        let result = self
            .users_service
            .lock()
            .expect("Failed to get users_service on signUp")
            .create_user(req.username, req.password);

        match result {
            Ok(_) => {
                let reply = SignUpResponse {
                    status_code: StatusCode::Success.into(),
                };
                Ok(Response::new(reply))
            }
            Err(_) => {
                let reply = SignUpResponse {
                    status_code: StatusCode::Failure.into(),
                };
                Ok(Response::new(reply))
            }
        }
    }

    async fn sign_out(
        &self,
        request: Request<SignOutRequest>,
    ) -> Result<Response<SignOutResponse>, Status> {
        let req = request.into_inner();

        self.sessions_service
            .lock()
            .expect("Failed to get sessions_service on signOut")
            .delete_session(&req.session_token);

        let reply = SignOutResponse {
            status_code: StatusCode::Success.into(),
        };
        Ok(Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sessions::SessionsImpl, users::UsersImpl};

    #[tokio::test]
    async fn sign_in_should_fail_if_user_not_found() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let session_service = Box::new(Mutex::new(SessionsImpl::default()));

        let auth_service = AuthService::new(users_service, session_service);

        let req = tonic::Request::new(SignInRequest {
            username: "123456".to_owned(),
            password: "654321".to_owned(),
        });

        let result = auth_service.sign_in(req).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Failure.into());
        assert!(result.user_uuid.is_empty());
        assert!(result.session_token.is_empty());
    }

    #[tokio::test]
    async fn sign_in_should_fail_if_incorrect_password() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let session_service = Box::new(Mutex::new(SessionsImpl::default()));
        let username = "mateus".to_owned();
        users_service
            .lock()
            .unwrap()
            .create_user(username.clone(), "password".to_owned())
            .unwrap();

        let auth_service = AuthService::new(users_service, session_service);

        let req = tonic::Request::new(SignInRequest {
            username,
            password: "wrong_pass".to_owned(),
        });

        let result = auth_service.sign_in(req).await.unwrap().into_inner();
        assert_eq!(result.status_code, StatusCode::Failure.into());
        assert!(result.session_token.is_empty());
        assert!(result.user_uuid.is_empty());
    }

    #[tokio::test]
    async fn sign_in_should_succeed() {
        let users_sevice = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));
        let username = "Mateus".to_owned();
        let password = "password".to_owned();
        users_sevice
            .lock()
            .unwrap()
            .create_user(username.clone(), password.clone())
            .unwrap();

        let auth_service = AuthService::new(users_sevice, sessions_service);

        let req = tonic::Request::new(SignInRequest { username, password });
        let result = auth_service.sign_in(req).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Success.into());
        assert_eq!(result.session_token.is_empty(), false);
        assert_eq!(result.user_uuid.is_empty(), false);
    }

    #[tokio::test]
    async fn sign_up_should_fail_if_username_exists() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));
        let username = "Mateus".to_owned();
        let password = "password".to_owned();
        users_service
            .lock()
            .unwrap()
            .create_user(username.clone(), password.clone())
            .unwrap();

        let auth_service = AuthService::new(users_service, sessions_service);
        let req = tonic::Request::new(SignUpRequest { username, password });
        let res = auth_service.sign_up(req).await.unwrap().into_inner();

        assert_eq!(res.status_code, StatusCode::Failure.into());
    }

    #[tokio::test]
    async fn sign_up_should_succeed() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

        let auth_service = AuthService::new(users_service, sessions_service);
        let username = "Mateus".to_owned();
        let password = "password".to_owned();
        let req = tonic::Request::new(SignUpRequest { username, password });
        let result = auth_service.sign_up(req).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Success.into());
    }

    #[tokio::test]
    async fn sign_out_should_succeed() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));
        let session_token = sessions_service.lock().unwrap().create_session("user_uuid");

        let auth_service = AuthService::new(users_service, sessions_service);
        let req = tonic::Request::new(SignOutRequest { session_token });
        let res = auth_service.sign_out(req).await.unwrap().into_inner();

        assert_eq!(res.status_code, StatusCode::Success.into());
    }
}
