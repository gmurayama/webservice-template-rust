use actix_web::{http::StatusCode, FromRequest, HttpRequest};
use std::{
    future::{ready, Ready},
    time::Duration,
};

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponseBuilder,
};
use futures_util::future::LocalBoxFuture;

#[derive(Clone)]
pub struct Timeout {
    duration: Duration,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Timeout { duration }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Timeout
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = TimeoutMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TimeoutMiddleware {
            service,
            duration: self.duration,
        }))
    }
}

pub struct TimeoutMiddleware<S> {
    service: S,
    duration: Duration,
}

impl<S, B> Service<ServiceRequest> for TimeoutMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let duration = self.duration;
        let request = req.request().clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = tokio::time::timeout(duration, fut).await;
            let r: ServiceResponse<EitherBody<B, BoxBody>> = match res {
                Ok(res) => {
                    let res = res?;
                    res.map_into_left_body()
                }
                Err(_) => ServiceResponse::new(
                    request,
                    HttpResponseBuilder::new(StatusCode::BAD_REQUEST).body("deadline"),
                )
                .map_into_right_body(),
            };

            Ok(r)
        })
    }
}
