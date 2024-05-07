use http::{header, HeaderMap, Request, Response};
use std::time::Duration;
use tower_http::{
    classify::{SharedClassifier, StatusInRangeAsFailures, StatusInRangeFailureClass},
    trace::{self, TraceLayer},
};
use tracing::{info, span, Level, Span};
use uuid::Uuid;

/// Creates a custom tracing span
#[derive(Clone, Copy, Debug)]
pub struct MakeSpan;

impl<B> trace::MakeSpan<B> for MakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let uri = request.uri();
        let span = span!(
            Level::INFO,
            "request",
            span.kind = "server",
            http.request.method = %request.method(),
            network.protocol.version = ?request.version(),
            uri.path = uri.path(),
            uri.scheme = uri.scheme_str().unwrap_or_default(),
            uri.query = uri.query().unwrap_or_default(),
            id = %Uuid::new_v4(),
        );

        let headers = request.headers();
        if let Some(user_agent) = headers
            .get(header::USER_AGENT)
            .map(|h| h.to_str().ok())
            .flatten()
        {
            span.record("user_agent.original", user_agent);
        }

        record_headers("request", &headers, &span);

        #[cfg(feature = "opentelemetry")]
        {
            use opentelemetry::propagation::TextMapPropagator;
            use opentelemetry_http::HeaderExtractor;
            use opentelemetry_sdk::propagation::{BaggagePropagator, TraceContextPropagator};
            use tracing_opentelemetry::OpenTelemetrySpanExt;

            let extractor = HeaderExtractor(headers);
            let context = TraceContextPropagator::new().extract(&extractor);
            let context = BaggagePropagator::new().extract_with_context(&context, &extractor);
            span.set_parent(context);
        }

        span
    }
}

/// Adds custom trace metadata from the request
#[derive(Clone, Copy, Debug)]
pub struct OnRequest;

impl<B> trace::OnRequest<B> for OnRequest {
    fn on_request(&mut self, _request: &Request<B>, _span: &Span) {
        info!("started processing request");
    }
}

/// Adds custom trace metadata from the request
#[derive(Clone, Copy, Debug)]
pub struct OnResponse;

impl<B> trace::OnResponse<B> for OnResponse {
    fn on_response(self, response: &Response<B>, _latency: Duration, span: &Span) {
        span.record("http.response.status", response.status().as_u16());
        record_headers("response", &response.headers(), span);
        info!("finished processing request");
    }
}

/// Adds custom trace metadata when a failure occurs
#[derive(Clone, Copy, Debug)]
pub struct OnFailure;

impl trace::OnFailure<StatusInRangeFailureClass> for OnFailure {
    fn on_failure(&mut self, class: StatusInRangeFailureClass, _latency: Duration, span: &Span) {
        if span.has_field("error") || span.has_field("error.type") {
            return;
        }

        span.record("error", true);

        let kind = match class {
            StatusInRangeFailureClass::StatusCode(status) => {
                if status.is_client_error() {
                    "client_error"
                } else {
                    "server_error"
                }
            }
            StatusInRangeFailureClass::Error(_) => "internal_error",
        };
        span.record("error.type", kind);
    }
}

/// Create a logging middleware layer
pub fn layer() -> TraceLayer<
    SharedClassifier<StatusInRangeAsFailures>,
    MakeSpan,
    OnRequest,
    OnResponse,
    (),
    (),
    OnFailure,
> {
    let classifier = StatusInRangeAsFailures::new_for_client_and_server_errors();
    TraceLayer::new(classifier.into_make_classifier())
        .make_span_with(MakeSpan)
        .on_request(OnRequest)
        .on_response(OnResponse)
        .on_failure(OnFailure)
        .on_eos(())
        .on_body_chunk(())
}

fn record_headers(direction: &'static str, headers: &HeaderMap, span: &Span) {
    for (header, value) in headers
        .iter()
        .filter_map(|(header, value)| value.to_str().ok().map(|value| (header, value)))
    {
        let header = header.as_str().to_lowercase();
        span.record(format!("http.{direction}.header.{header}").as_str(), value);
    }
}
