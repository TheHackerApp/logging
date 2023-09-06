# Logging

The shared logging setup for The Hacker App services.

Uses the [`tracing`][tracing] ecosystem to provide logging and application tracing. Provides a console exporter using
[`tracing-subscriber`][tracing-subscriber] and optional [OpenTelemetry][otel] support using 
[`tracing-opentelemetry`][tracing-otel]. More detailed logging can be configured using the `RUST_LOG` environment
variable using [tracing directives][].

[tracing]: https://docs.rs/tracing/latest/tracing/
[tracing-subscriber]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/
[otel]: https://docs.rs/opentelemetry/latest/opentelemetry/
[tracing-otel]: https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/
[tracing directives]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
