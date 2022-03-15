extern crate opentelemetry;
extern crate opentelemetry_otlp;
extern crate slog_opentelemetry;
#[macro_use]
extern crate slog;
extern crate tokio;

use opentelemetry::{
    sdk::{log::LogEmitterProvider, trace::TracerProvider},
    trace::{Tracer, TracerProvider as APITracerProvider},
};
use opentelemetry_otlp::{
    new_exporter, LogExporterBuilder, SpanExporterBuilder, TonicExporterBuilder, WithExportConfig,
};
use slog::Drain;
use std::{
    sync::Mutex,
    time::{Duration, SystemTime},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_exporter = LogExporterBuilder::Tonic(tonic_builder()).build_log_exporter()?;
    let trace_exporter = SpanExporterBuilder::Tonic(tonic_builder()).build_span_exporter()?;

    let mut emitter_provider = LogEmitterProvider::builder()
        .with_simple_exporter(log_exporter)
        .build();

    let tracer_provider = TracerProvider::builder()
        .with_simple_exporter(trace_exporter)
        .build();
    {
        let drain =
            slog_opentelemetry::OpenTelemetryBuilder::new(SystemTime::now).build(&emitter_provider);
        let root = slog::Logger::root_typed(Mutex::new(drain).map(slog::Fuse), o!());

        let tracer = tracer_provider.tracer("foo");
        tracer.in_span("doing_work", |_cx| {
             info!(root, "without fields");
	    info!(root, "info record with fields"; "field1" => "lorem impsum", "field2" => 1, "field3" => true, "field4" => 3.41);
	    debug!(root, "debug record with fields"; "field1" => "lorem impsum", "field2" => 1, "field3" => true, "field4" => 3.41);
	    trace!(root, "trace record with fields"; "field1" => "lorem impsum", "field2" => 1, "field3" => true, "field4" => 3.41);
	    error!(root, "error record with fields"; "field1" => "lorem impsum", "field2" => 1, "field3" => true, "field4" => 3.41);
	    crit!(root, "critical/fatal record with fields"; "field1" => "lorem impsum", "field2" => 1, "field3" => true, "field4" => 3.41)
        });
    }

    emitter_provider.shutdown();
    Ok(())
}

fn tonic_builder() -> TonicExporterBuilder {
    new_exporter()
        .tonic()
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_endpoint("http://127.0.0.1:4317")
        .with_timeout(Duration::from_secs(1))
}
