extern crate opentelemetry;
extern crate slog_opentelemetry;
#[macro_use]
extern crate slog;

use opentelemetry::{
    sdk::{
        export::{log, trace},
        log::LogEmitterProvider,
        trace::TracerProvider,
    },
    trace::{Tracer, TracerProvider as APITracerProvider},
};

use slog::Drain;
use std::{io::stdout, sync::Mutex, time::SystemTime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_exporter = log::stdout::Exporter::new(stdout(), true);
    let mut emitter_provider = LogEmitterProvider::builder()
        .with_simple_exporter(log_exporter)
        .build();

    let tracer_provider = TracerProvider::builder()
        .with_simple_exporter(trace::stdout::Exporter::new(stdout(), true))
        .build();
    let tracer = tracer_provider.tracer("foo");

    {
        let drain =
            slog_opentelemetry::OpenTelemetryBuilder::new(SystemTime::now).build(&emitter_provider);
        let root = slog::Logger::root_typed(Mutex::new(drain).map(slog::Fuse), o!());

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
