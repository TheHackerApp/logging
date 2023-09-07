use eyre::WrapErr;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::{Directive, EnvFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};

pub use opentelemetry_otlp::Protocol as OpenTelemetryProtocol;

mod otel;

/// OpenTelemetry exporter configuration
#[derive(Debug)]
pub struct OpenTelemetry<'o> {
    url: &'o str,
    protocol: OpenTelemetryProtocol,
}

impl<'o> OpenTelemetry<'o> {
    /// Create a new OpenTelemetry configuration
    pub fn new(url: Option<&'o str>, protocol: OpenTelemetryProtocol) -> Option<Self> {
        url.map(|url| Self { url, protocol })
    }
}

/// Setup logging and error reporting
pub fn init<D>(default_directive: D, opentelemetry: Option<OpenTelemetry<'_>>) -> eyre::Result<()>
where
    D: Into<Directive>,
{
    let registry = Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(default_directive.into())
                .from_env_lossy(),
        )
        .with(ErrorLayer::default())
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_target(true),
        );

    if let Some(opentelemetry) = opentelemetry {
        let exporter = otel::exporter(opentelemetry.protocol, opentelemetry.url);
        let tracer =
            otel::tracer(exporter).wrap_err("failed to initialize OpenTelemetry tracer")?;

        let opentelemetry = tracing_opentelemetry::layer()
            .with_location(true)
            .with_tracked_inactivity(true)
            .with_exception_field_propagation(true)
            .with_exception_fields(true)
            .with_tracer(tracer);

        registry.with(opentelemetry).init();
    } else {
        registry.init();
    }

    Ok(())
}
