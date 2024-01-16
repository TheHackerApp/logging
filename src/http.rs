use http::Request;
#[cfg(feature = "opentelemetry")]
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer},
};
use tracing::{span, Level, Span};
use uuid::Uuid;

/// Creates a custom tracing span
#[derive(Clone, Debug, Default)]
pub struct MakeSpanWithId {
    #[cfg(feature = "opentelemetry")]
    propagator: TraceContextPropagator,
}

impl<B> MakeSpan<B> for MakeSpanWithId {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let span = span!(
            Level::INFO,
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            id = %Uuid::new_v4(),
        );

        #[cfg(feature = "opentelemetry")]
        {
            use opentelemetry::propagation::TextMapPropagator;
            use opentelemetry_http::HeaderExtractor;
            use tracing_opentelemetry::OpenTelemetrySpanExt;

            let context = self.propagator.extract(&HeaderExtractor(request.headers()));
            span.set_parent(context);
        }

        span
    }
}

/// Create a logging middleware layer
pub fn layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MakeSpanWithId> {
    TraceLayer::new_for_http()
        .make_span_with(MakeSpanWithId::default())
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}
