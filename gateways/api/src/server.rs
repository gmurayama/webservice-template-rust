use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::net::TcpListener;
use std::sync::Mutex;
use tracing::log;

use crate::middlewares::{metrics::Metrics, tracing::Tracing};

#[tracing::instrument]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().finish()
}

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
    pub app: AppSettings,
    pub metrics: MetricSettings,
}

pub struct AppSettings {
    pub host: String,
    pub port: u16,
}

pub struct MetricSettings {
    pub host: String,
    pub port: u16,
    pub registry: Registry,
}

pub struct Server {
    port: u16,
    server: actix_web::dev::Server,
    metrics_server: actix_web::dev::Server,
}

impl Server {
    pub fn setup(settings: Settings) -> eyre::Result<Server> {
        let listener = TcpListener::bind(format!("{}:{}", settings.app.host, settings.app.port))?;
        let metrics_listener = TcpListener::bind(format!(
            "{}:{}",
            settings.metrics.host, settings.metrics.port
        ))?;

        let port = listener.local_addr().unwrap().port();

        let mut registry = settings.metrics.registry;
        let api_metrics = Metrics::new(&mut registry);

        let state = AppState { registry };
        let state = web::Data::new(Mutex::new(state));

        let server = HttpServer::new(move || {
            App::new()
                .wrap(Tracing::middleware())
                .wrap(api_metrics.clone())
                .route("/healthcheck", web::get().to(healthcheck))
        })
        .listen(listener)
        .map(|s| {
            log::info!(
                "Started listening on {}:{}",
                settings.app.host,
                settings.app.port
            );

            s
        })?
        .run();

        let metrics_server = HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .route("/metrics", web::get().to(metrics_handler))
        })
        .listen(metrics_listener)
        .map(|s| {
            log::info!(
                "Metrics Server listening on {}:{}",
                settings.metrics.host,
                settings.metrics.port
            );

            s
        })?
        .run();

        let server = Server {
            port,
            server,
            metrics_server,
        };

        Ok(server)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let result = futures_util::join!(self.metrics_server, self.server);

        match result {
            (Err(e1), Err(e2)) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{}\n{}", e1, e2),
            )),
            (Ok(_), Err(err)) => Err(err),
            (Err(err), Ok(_)) => Err(err),
            _ => Ok(()),
        }
    }
}
