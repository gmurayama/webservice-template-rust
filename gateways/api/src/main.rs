use api::{
    metrics::ApiMetrics,
    server::{self, setup_server},
};
use infrastructure::{
    self,
    telemetry::{self, JaegerSettings, LoggingSettings},
};
use prometheus_client::registry::Registry;
use std::net::TcpListener;

mod settings;
use settings::get_config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let settings = get_config()?;

    telemetry::setup(telemetry::Settings {
        log: LoggingSettings {
            format: telemetry::LoggingOptions::PrettyPrint,
        },
        jaeger: JaegerSettings {
            host: settings.jaeger.host,
            port: settings.jaeger.port,
            sampler_param: settings.jaeger.sampler_param,
        },
        service_name: settings.app.service_name,
    });

    let mut registry = <Registry>::default();
    let api_metrics = ApiMetrics::new(&mut registry);

    let listener = TcpListener::bind(format!("{}:{}", settings.app.host, settings.app.port))?;
    let server = setup_server(server::Settings {
        listener,
        metrics: api_metrics,
        registry,
    })?;

    server.await?;

    telemetry::teardown().await;

    Ok(())
}
