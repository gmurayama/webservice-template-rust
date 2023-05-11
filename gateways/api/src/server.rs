use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::net::TcpListener;
use std::sync::Mutex;
use tracing::log;

use crate::{
    metrics::ApiMetrics,
    middlewares::{metrics::Metrics, tracing::Tracing},
};

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
    pub host: String,
    pub port: u16,
    pub registry: Registry,
}

pub struct Server {
    port: u16,
    server: actix_web::dev::Server,
}

impl Server {
    pub fn setup(settings: Settings) -> eyre::Result<Server> {
        let listener = TcpListener::bind(format!("{}:{}", settings.host, settings.port))?;
        let port = listener.local_addr().unwrap().port();

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
                .wrap(Tracing::new())
                .service(healthcheck)
                .service(metrics_handler)
        })
        .listen(listener)
        .and_then(|s| {
            log::info!("Started listening on {}:{}", settings.host, settings.port);
            Ok(s)
        })?
        .run();

        let server = Server { port, server };

        Ok(server)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
