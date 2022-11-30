use infrastructure::{
    self,
    telemetry::{self, JaegerSettings, LoggingSettings, Settings},
};
use std::net::TcpListener;

mod server;
use server::setup_server;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry::setup(Settings {
        log: LoggingSettings {
            format: telemetry::LoggingOptions::PrettyPrint,
        },
        jaeger: JaegerSettings {
            host: "127.0.0.1".to_string(),
            port: 6831,
            sampling_percentage: 1.0,
        },
        service_name: "api".to_string(),
    });

    let listener = TcpListener::bind("127.0.0.1:7000")?;
    let server = setup_server(listener).await?;
    server.await?;

    telemetry::teardown().await;

    Ok(())
}
