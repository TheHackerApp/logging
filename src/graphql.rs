use async_graphql::{
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextResolve,
        NextValidation, ResolveInfo,
    },
    parser::types::{ExecutableDocument, OperationType, Selection},
    Response, ServerError, ServerResult, ValidationResult, Value, Variables,
};
use std::sync::Arc;
use tracing::{span, warn, Instrument, Level, Span};

/// Adds tracing to the GraphQL flow
///
/// Adapted from the [`async_graphql::extensions::Tracing`](https://docs.rs/async-graphql/latest/async_graphql/extensions/struct.Tracing.html)
/// and [`async_graphql::extensions::Logger`](https://docs.rs/async-graphql/latest/async_graphql/extensions/struct.Logger.html)
/// extensions.
#[derive(Clone, Copy, Debug)]
pub struct GraphQL;

impl ExtensionFactory for GraphQL {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(LoggingExtension)
    }
}

struct LoggingExtension;

#[async_trait::async_trait]
impl Extension for LoggingExtension {
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let span = span!(Level::INFO, "parse");

        async move {
            let document = next.run(ctx, query, variables).await?;

            let is_schema = document
                .operations
                .iter()
                .filter(|(_, operation)| operation.node.ty == OperationType::Query)
                .any(|(_, operation)| operation.node.selection_set.node.items.iter().any(|selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));
            if !is_schema {
                let span = Span::current();
                span.record("graphql.document", ctx.stringify_execute_doc(&document, variables));
            }

            Ok(document)
        }
            .instrument(span)
            .await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> async_graphql::Result<ValidationResult, Vec<ServerError>> {
        let span = span!(Level::INFO, "validation");
        async move {
            let result = next.run(ctx).await;

            if let Ok(result) = result {
                let span = Span::current();
                span.record("graphql.complexity", result.complexity);
                span.record("graphql.depth", result.depth);
            }

            result
        }
        .instrument(span)
        .await
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let span = span!(Level::INFO, "execute", graphql.operation.name = %operation_name.unwrap_or_default());

        next.run(ctx, operation_name).instrument(span).await
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        let path_node = info.path_node.to_string();
        if path_node.starts_with("__schema") {
            return next.run(ctx, info).await;
        }

        let span = span!(
            Level::INFO, "field",
            graphql.node.path = %path_node,
            graphql.node.parent_type = %info.parent_type,
            graphql.node.return_type = %info.return_type,
        );

        async move {
            let result = next.run(ctx, info).await;

            // we do the actual error handling elsewhere, these errors are typically caused by
            // user error.
            if let Err(ref err) = result {
                let span = Span::current();
                span.record("error", true);
                span.record("error.type", "graphql");
                warn!(error = %err.message, locations = ?err.locations, path = ?err.path);
            }

            result
        }
        .instrument(span)
        .await
    }
}
