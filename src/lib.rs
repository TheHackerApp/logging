use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::{Directive, EnvFilter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};

/// Setup logging and error reporting
pub fn init<D>(default_directive: D)
where
    D: Into<Directive>,
{
    Registry::default()
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
        )
        .init();
}
