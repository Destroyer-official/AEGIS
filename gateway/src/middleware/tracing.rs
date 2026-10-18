use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::{
    global,
    trace::{Span, SpanKind, Status, Tracer},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{resource::Resource, runtime, trace::TracerProvider};
use std::time::Instant;
use tracing::info;

pub fn init_tracing(service_name: &str) -> TracerProvider {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to create OTLP span exporter");

    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
    ]);

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(resource)
        .build();

    global::set_tracer_provider(provider.clone());
    provider
}

pub async fn trace_middleware(request: Request, next: Next) -> Response {
    let tracer = global::tracer("aegis-gateway");
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let client_ip = request
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|a| a.to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let start = Instant::now();

    let mut span = tracer
        .span_builder(format!("{} {}", method, uri))
        .with_kind(SpanKind::Server)
        .with_attributes(vec![
            KeyValue::new("http.method", method.clone()),
            KeyValue::new("http.url", uri.clone()),
            KeyValue::new("http.client_ip", client_ip.clone()),
            KeyValue::new(
                "http.user_agent",
                request
                    .headers()
                    .get("user-agent")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string(),
            ),
        ])
        .start(&tracer);

    let response = next.run(request).await;

    let elapsed = start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();

    span.set_attribute(KeyValue::new("http.status_code", status as i64));
    span.set_attribute(KeyValue::new("http.response_time_ms", elapsed as i64));

    if status >= 400 {
        span.set_status(Status::Error {
            description: format!("HTTP {}", status).into(),
        });
    } else {
        span.set_status(Status::Ok);
    }

    span.end();

    info!(
        method = %method,
        status = status,
        uri = %uri,
        response_time_ms = elapsed,
        client_ip = %client_ip,
        "request completed"
    );

    response
}
