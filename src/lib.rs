use eyre::WrapErr;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::{Directive, EnvFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};

pub use opentelemetry_otlp::Protocol as OpenTelemetryProtocol;

mod otel;

/// Create a new logging configurator
pub fn config() -> Builder<'static> {
    Builder::default()
}

/// Configure options for logging and application tracing
#[derive(Debug)]
pub struct Builder<'b> {
    default_directive: Directive,
    opentelemetry_url: Option<&'b str>,
    opentelemetry_protocol: OpenTelemetryProtocol,
}

impl<'b> Builder<'b> {
    /// Set the default directive being used
    pub fn default_directive<D>(mut self, directive: D) -> Self
    where
        D: Into<Directive>,
    {
        self.default_directive = directive.into();
        self
    }

    /// Configure the OpenTelemetry exporter
    pub fn opentelemetry(mut self, protocol: OpenTelemetryProtocol, url: &'b str) -> Self {
        self.opentelemetry_url = Some(url);
        self.opentelemetry_protocol = protocol;
        self
    }

    /// Install the logger and error reporting
    pub fn init(self) -> eyre::Result<()> {
        let registry = Registry::default()
            .with(
                EnvFilter::builder()
                    .with_default_directive(self.default_directive)
                    .from_env_lossy(),
            )
            .with(ErrorLayer::default())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_file(true)
                    .with_line_number(true)
                    .with_target(true),
            );

        if let Some(url) = self.opentelemetry_url {
            let exporter = otel::exporter(self.opentelemetry_protocol, url);
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
}

impl<'b> Default for Builder<'b> {
    fn default() -> Self {
        Self {
            default_directive: Level::INFO.into(),
            opentelemetry_url: None,
            opentelemetry_protocol: OpenTelemetryProtocol::Grpc,
        }
    }
}
