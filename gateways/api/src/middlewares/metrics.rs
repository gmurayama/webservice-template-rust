use std::{
    future::{ready, Ready},
    sync::Arc,
    time::Instant,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use prometheus_client::metrics::{counter::Counter, family::Family, histogram::Histogram};

pub struct Metrics {
    request_duration: Family<Vec<(String, String)>, Histogram>,
    request_count: Family<Vec<(String, String)>, Counter>,
}

impl Metrics {
    pub fn new(
        request_duration: Family<Vec<(String, String)>, Histogram>,
        request_count: Family<Vec<(String, String)>, Counter>,
    ) -> Self {
        Metrics {
            request_duration,
            request_count,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Metrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddleware {
            service,
            request_duration: Arc::new(self.request_duration.clone()),
            request_count: Arc::new(self.request_count.clone()),
        }))
    }
}

pub struct MetricsMiddleware<S> {
    service: S,
    request_duration: Arc<Family<Vec<(String, String)>, Histogram>>,
    request_count: Arc<Family<Vec<(String, String)>, Counter>>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let now = Instant::now();
        let path = req.path().to_string();
        let method = req.method().to_string();

        let request_duration = self.request_duration.clone();
        let request_count = self.request_count.clone();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            let elapsed = now.elapsed().as_millis() as f64;

            request_duration
                .get_or_create(&vec![(method.clone(), path.clone())])
                .observe(elapsed);

            request_count.get_or_create(&vec![(method, path)]).inc();

            Ok(res)
        })
    }
}
