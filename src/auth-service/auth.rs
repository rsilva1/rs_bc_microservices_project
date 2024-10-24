use std::{borrow::BorrowMut, sync::Mutex};

use authentication::{auth_server::Auth, SignInRequest, SignInResponse, SignOutRequest, SignOutResponse, SignUpRequest, SignUpResponse, StatusCode};
use tonic::{Request, Response, Status};

use crate::{sessions::Sessions, users::Users};

pub mod authentication {
    tonic::include_proto!("authentication");
}

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

        let users_service = self.users_service.lock()
            .expect("Users service mutex poisoned");

        let user_uuid = users_service.get_user_uuid(req.username, req.password);
        if user_uuid.is_none() {
            return Ok(Response::new(SignInResponse {
                status_code: StatusCode::Failure.into(),
                user_uuid: "".to_owned(),
                session_token: "".to_owned(),
            }))
        }
        let user_uuid = user_uuid.unwrap();

        let session_token = self.sessions_service.lock()
            .expect("Sessions service mutex poisoned")
            .borrow_mut()
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
        request: Request<SignUpRequest>
    ) -> Result<Response<SignUpResponse>, Status> {
        println!("Got a request: {:?}", request);

        let req = request.into_inner();

        let mut users_service = self.users_service.lock()
            .expect("Users service mutex poisoned");

        let result = users_service.borrow_mut().create_user(req.username, req.password);

        match result {
            Ok(_) => {
                return Ok(Response::new(SignUpResponse {
                    status_code: StatusCode::Success.into()
                }));
            },
            Err(_) => {
                return Ok(Response::new(SignUpResponse {
                    status_code: StatusCode::Failure.into()
                }));
            }
        }
    }

    async fn sign_out(
        &self,
        request: Request<SignOutRequest>
    ) -> Result<Response<SignOutResponse>, Status> {
        println!("Got a request: {:?}", request);

        let req = request.into_inner();

        self.sessions_service.lock()
            .expect("Sessions service mutex poisoned")
            .borrow_mut()
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

    use authentication::{SignOutRequest, SignUpRequest, StatusCode};

    use crate::{sessions::SessionsImpl, users::UsersImpl};

    #[tokio::test]
    async fn sign_in_should_fail_if_user_not_found() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

        let auth_service = AuthService::new(users_service, sessions_service);

        let request = Request::new(SignInRequest {
            username: "username".to_owned(),
            password: "password".to_owned(),
        });

        let result = auth_service.sign_in(request).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Failure.into());
        assert!(result.user_uuid.is_empty());
        assert!(result.session_token.is_empty());
    }

    #[tokio::test]
    async fn sign_in_should_fail_if_incorrect_password() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

        users_service.lock().unwrap().create_user(
            "username".to_owned(),
            "password".to_owned(),
        ).expect("should create user");

        let auth_service = AuthService::new(users_service, sessions_service);

        let request = Request::new(SignInRequest {
            username: "username".to_owned(),
            password: "false_password".to_owned(),
        });

        let result = auth_service.sign_in(request).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Failure.into());
        assert!(result.user_uuid.is_empty());
        assert!(result.session_token.is_empty());
    }

    #[tokio::test]
    async fn sign_in_should_succeed() {
        let mut users_service = UsersImpl::default();

        users_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        let user_uuid = users_service.get_user_uuid(
            "username".to_owned(),
            "password".to_owned())
            .expect("should get user uuid");

        let users_service = Box::new(Mutex::new(users_service));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

        let auth_service = AuthService::new(users_service, sessions_service);

        let request = Request::new(SignInRequest {
            username: "username".to_owned(),
            password: "password".to_owned(),
        });

        let result = auth_service.sign_in(request).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Success.into());
        assert_eq!(result.user_uuid, user_uuid);
        assert_eq!(result.session_token.is_empty(), false);
    }

    #[tokio::test]
    async fn sign_up_should_fail_if_username_exists() {
        let mut users_service = UsersImpl::default();

        users_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        let users_service = Box::new(Mutex::new(users_service));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

        let auth_service = AuthService::new(users_service, sessions_service);

        let request = Request::new(SignUpRequest {
            username: "username".to_owned(),
            password: "password".to_owned()
        });

        let result = auth_service.sign_up(request).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Failure.into());
    }

    #[tokio::test]
    async fn sign_up_should_succeed() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

        let auth_service = AuthService::new(users_service, sessions_service);

        let request = Request::new(SignUpRequest {
            username: "username".to_owned(),
            password: "password".to_owned(),
        });

        let result = auth_service.sign_up(request).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Success.into());
    }

    #[tokio::test]
    async fn sign_out_should_succeed() {
        let users_service = Box::new(Mutex::new(UsersImpl::default()));
        let sessions_service = Box::new(Mutex::new(SessionsImpl::default()));

        let auth_service = AuthService::new(users_service, sessions_service);

        let request = tonic::Request::new(SignOutRequest {
            session_token: "".to_owned()
        });

        let result = auth_service.sign_out(request).await.unwrap();

        assert_eq!(result.into_inner().status_code, StatusCode::Success.into());
    }

    #[tokio::test]
    async fn sign_out_should_succeed_when_given_valid_session() {
        let mut users_service = UsersImpl::default();
        users_service.create_user(
            "username".to_owned(),
            "password".to_owned())
            .expect("should create user");
        let user_uuid = users_service.get_user_uuid(
            "username".to_owned(),
            "password".to_owned())
            .expect("should get user uuid");

        let users_service = Box::new(Mutex::new(users_service));
        let mut sessions_service = SessionsImpl::default();

        let session_token = sessions_service.create_session(&user_uuid);

        let sessions_service = Box::new(Mutex::new(sessions_service));

        let auth_service = AuthService::new(users_service, sessions_service);

        let request = Request::new(SignOutRequest { session_token });

        let result = auth_service.sign_out(request).await.unwrap().into_inner();

        assert_eq!(result.status_code, StatusCode::Success.into());
    }
}
