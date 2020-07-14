use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{
    dev::ServiceRequest,
    dev::ServiceResponse,
    error::ErrorUnauthorized,
    http::{HeaderName, HeaderValue},
    Error,
};
use futures::future::{self, ok, Ready};
use futures::Future;

const TOKEN_HEADER: &'static str = "X-ACCESS-TOKEN";

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct UserAuth;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for UserAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UserAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UserAuthMiddleware { service })
    }
}

pub struct UserAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service for UserAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());

        match get_user_id_from_token(&req) {
            Ok(user_id) => {
                let name = HeaderName::from_static("user-id");
                //TODO: add error handling here
                let value = HeaderValue::from_str(&user_id).unwrap();
                req.headers_mut().append(name, value);
                Box::pin(self.service.call(req))
            }
            Err(err) => {
                eprintln!("{}", err);
                let fut = future::err(ErrorUnauthorized("not authorized"));
                Box::pin(fut)
            }
        }
    }
}

fn get_user_id_from_token(req: &ServiceRequest) -> Result<String, anyhow::Error> {
    let token_str = req
        .headers()
        .get(TOKEN_HEADER)
        .ok_or(anyhow!("Missing attribute : {}", TOKEN_HEADER))?
        .to_str()?;
    super::user_actions::decrypt_token(token_str)
}
