use std::time::Duration;

use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::Sampler, Resource};
use tokio::task;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{
    filter::filter_fn, fmt::format::FmtSpan, prelude::__tracing_subscriber_SubscriberExt,
    EnvFilter, Layer, Registry,
};

#[derive(PartialEq)]
pub enum LoggingOptions {
    PrettyPrint,
    JSON,
}

pub struct TelemetrySettings {
    pub host: String,
    pub port: u32,
    pub sampler_param: f64,
}

pub struct LoggingSettings {
    pub format: LoggingOptions,
}

pub struct Settings {
    pub log: LoggingSettings,
    pub telemetry: TelemetrySettings,
    pub service_name: String,
}

pub fn setup(settings: Settings) {
    LogTracer::init().expect("Failed to set logger");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let emit_bunyan = settings.log.format == LoggingOptions::JSON;
    let bunyan_json_layer = JsonStorageLayer.with_filter(filter_fn(move |_| emit_bunyan));
    let bunyan_formatting_layer =
        BunyanFormattingLayer::new(settings.service_name.clone(), std::io::stdout)
            .with_filter(filter_fn(move |_| emit_bunyan));

    let emit_pretty_formating = settings.log.format == LoggingOptions::PrettyPrint;
    let pretty_formatting_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::NEW)
        .with_filter(filter_fn(move |_| emit_pretty_formating));

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(format!(
                    "http://{}:{}",
                    settings.telemetry.host, settings.telemetry.port
                ))
                .with_timeout(Duration::from_secs(2)),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(settings.telemetry.sampler_param))
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    settings.service_name.clone(),
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(bunyan_json_layer)
        .with(bunyan_formatting_layer)
        .with(pretty_formatting_layer)
        .with(telemetry);

    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub async fn teardown() {
    let res = task::spawn_blocking(move || {
        global::shutdown_tracer_provider();
    });

    if let Err(_) = tokio::time::timeout(Duration::from_secs(5), res).await {
        log::error!("could not shutdown tracer provider in 5 sec");
    }
}
