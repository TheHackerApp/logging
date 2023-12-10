use opentelemetry::trace::TraceError;
use opentelemetry_otlp::{Protocol, SpanExporterBuilder, WithExportConfig};
use opentelemetry_sdk::{
    resource::{
        EnvResourceDetector, OsResourceDetector, ProcessResourceDetector,
        SdkProvidedResourceDetector, TelemetryResourceDetector,
    },
    trace::{self, Tracer},
    Resource,
};
use std::time::Duration;

/// Configuration for the exporter
#[derive(Debug)]
pub(crate) struct Config<'c> {
    pub(crate) url: &'c str,
    pub(crate) protocol: Protocol,
}

/// Create a new tracing pipeline
pub(crate) fn tracer(exporter: SpanExporterBuilder) -> Result<Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace_config())
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

/// Create a new span exporter depending on the protocol
pub(crate) fn exporter(config: Config<'_>) -> SpanExporterBuilder {
    match config.protocol {
        Protocol::Grpc => opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(config.url)
            .into(),
        Protocol::HttpBinary => opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(config.url)
            .into(),
    }
}

/// Create a new tracing configuration
fn trace_config() -> trace::Config {
    trace::config().with_resource(resource_detectors())
}

/// Setup resource detectors to populate environment
fn resource_detectors() -> Resource {
    Resource::from_detectors(
        Duration::from_secs(5),
        vec![
            Box::new(SdkProvidedResourceDetector),
            Box::<EnvResourceDetector>::default(),
            Box::new(OsResourceDetector),
            Box::new(ProcessResourceDetector),
            Box::new(TelemetryResourceDetector),
        ],
    )
}
