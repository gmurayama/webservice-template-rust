use api::server;
use infrastructure::{self, telemetry};
use prometheus_client::registry::Registry;

mod settings;
use settings::get_config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let settings = get_config()?;

    telemetry::setup(telemetry::Settings {
        log: telemetry::LoggingSettings {
            format: telemetry::LoggingOptions::PrettyPrint,
        },
        jaeger: telemetry::JaegerSettings {
            host: settings.jaeger.host,
            port: settings.jaeger.port,
            sampler_param: settings.jaeger.sampler_param,
        },
        service_name: settings.app.service_name,
    });

    let registry = Registry::default();

    let server = server::Server::setup(server::Settings {
        app: server::AppSettings {
            host: settings.app.host,
            port: settings.app.port,
        },
        metrics: server::MetricSettings {
            host: settings.metric.host,
            port: settings.metric.port,
            registry,
        },
    })?;

    server.run().await?;

    telemetry::teardown().await;

    Ok(())
}
