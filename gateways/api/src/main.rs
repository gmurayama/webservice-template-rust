use infrastructure::{
    self,
    telemetry::{self, JaegerSettings, LoggingSettings, Settings},
};
use std::net::TcpListener;

mod server;
use server::setup_server;

mod settings;
use settings::get_config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let settings = get_config()?;

    telemetry::setup(Settings {
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

    let listener = TcpListener::bind("127.0.0.1:7000")?;
    let server = setup_server(listener).await?;
    server.await?;

    telemetry::teardown().await;

    Ok(())
}
