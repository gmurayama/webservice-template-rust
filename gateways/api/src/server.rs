use actix_web::{dev::Server, get, web, App, HttpResponse, HttpServer, Responder};
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::net::TcpListener;
use std::sync::Mutex;
use tracing_actix_web::TracingLogger;

use crate::{metrics::ApiMetrics, middlewares::metrics::Metrics};

#[tracing::instrument]
#[get("/healthcheck")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/metrics")]
async fn metrics_handler(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let mut body = String::new();
    encode(&mut body, &state.registry).unwrap();
    HttpResponse::Ok()
        .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
        .body(body)
}

pub struct AppState {
    pub registry: Registry,
}

pub struct Settings {
    pub metrics: ApiMetrics,
    pub listener: TcpListener,
    pub registry: Registry,
}

pub fn setup_server(settings: Settings) -> eyre::Result<Server> {
    let state = AppState {
        registry: settings.registry,
    };

    let state = web::Data::new(Mutex::new(state));

    let server = HttpServer::new(move || {
        let metrics = settings.metrics.clone();

        App::new()
            .app_data(state.clone())
            .wrap(Metrics::new(
                metrics.request_duration,
                metrics.request_count,
            ))
            .wrap(TracingLogger::default())
            .service(healthcheck)
            .service(metrics_handler)
    })
    .listen(settings.listener)?
    .run();

    Ok(server)
}
