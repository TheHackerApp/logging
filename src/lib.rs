use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::{Directive, EnvFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};

#[cfg(feature = "opentelemetry")]
pub use opentelemetry_otlp::Protocol as OpenTelemetryProtocol;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "opentelemetry")]
mod otel;

/// Create a new logging configurator
pub fn config() -> Builder<'static> {
    Builder::default()
}

/// Configure options for logging and application tracing
#[derive(Debug)]
pub struct Builder<'b> {
    default_directive: Directive,
    #[cfg(feature = "opentelemetry")]
    opentelemetry: Option<otel::Config<'b>>,
    #[cfg(not(feature = "opentelemetry"))]
    _phantom: std::marker::PhantomData<&'b ()>,
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
    #[cfg(feature = "opentelemetry")]
    pub fn opentelemetry(mut self, protocol: OpenTelemetryProtocol, url: &'b str) -> Self {
        self.opentelemetry = Some(otel::Config { protocol, url });
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

        #[cfg(feature = "opentelemetry")]
        if let Some(config) = self.opentelemetry {
            use eyre::WrapErr;

            let exporter = otel::exporter(config);
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

        #[cfg(not(feature = "opentelemetry"))]
        registry.init();

        Ok(())
    }
}

impl<'b> Default for Builder<'b> {
    fn default() -> Self {
        Self {
            default_directive: Level::INFO.into(),
            #[cfg(feature = "opentelemetry")]
            opentelemetry: None,
            #[cfg(not(feature = "opentelemetry"))]
            _phantom: std::marker::PhantomData::default(),
        }
    }
}
