use lazy_static::lazy_static;
use opentelemetry::{
    log::LogError,
    sdk::log::{Any, LogEmitter, LogEmitterProvider, LogRecord, Severity},
    Context,
};
use opentelemetry_semantic_conventions::trace;
use slog::{Drain, Key, Level, OwnedKVList, Record, Serializer, KV};
use std::{borrow::Cow, collections::BTreeMap, fmt::Arguments, time::SystemTime};

#[cfg(feature = "nested-values")]
use slog::SerdeValue;

/// OpenTelemetry `Drain`. Each record is emitted to the configured
/// `LogEmitter`.
#[derive(Debug)]
pub struct OpenTelemetry<F: Fn() -> SystemTime> {
    emitter: LogEmitter,
    resource: Option<BTreeMap<Cow<'static, str>, Any>>,
    timestamp: F,
}

const LOG_EMITTER_NAME: &str = "github.com/vibhavp/slog-opentelemetry";

/// A builder for the [`OpenTelemetry`] `Drain`.
#[derive(Debug)]
pub struct OpenTelemetryBuilder<F: Fn() -> SystemTime> {
    resource: Option<BTreeMap<Cow<'static, str>, Any>>,
    timestamp: F,
}

impl<F> OpenTelemetryBuilder<F>
where
    F: Fn() -> SystemTime,
{
    /// Create a new builder with a function that returns the current timestamp
    pub fn new(f: F) -> Self {
        Self {
            timestamp: f,
            resource: None,
        }
    }

    /// Set resource for all emitted Records.
    pub fn with_resource(self, resource: BTreeMap<Cow<'static, str>, Any>) -> Self {
        Self {
            resource: Some(resource),
            ..self
        }
    }

    /// Build a `Drain`, consuming this builder.
    pub fn build(self, emitter_provider: &LogEmitterProvider) -> OpenTelemetry<F> {
        OpenTelemetry {
            emitter: emitter_provider
                .versioned_log_emitter(LOG_EMITTER_NAME, Some(env!("CARGO_PKG_VERSION"))),
            resource: self.resource,
            timestamp: self.timestamp,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Slog(#[from] slog::Error),
    #[error(transparent)]
    Otel(#[from] LogError),
}

lazy_static! {
    static ref CODE_LINENO: String = trace::CODE_LINENO.to_string();
    static ref CODE_FILEPATH: String = trace::CODE_FILEPATH.to_string();
    static ref CODE_FUNCTION: String = trace::CODE_FUNCTION.to_string();
    static ref CODE_NAMESPACE: String = trace::CODE_NAMESPACE.to_string();
}

impl<F: Fn() -> SystemTime> Drain for OpenTelemetry<F> {
    type Ok = ();
    type Err = Error;
    fn log(&self, record: &Record<'_>, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        let context = Context::current();
        let mut attrs = OtelSerializer(BTreeMap::new());

        attrs
            .0
            .insert(Cow::Borrowed(&CODE_LINENO), record.line().into());
        attrs
            .0
            .insert(Cow::Borrowed(&CODE_FILEPATH), record.file().into());
        attrs
            .0
            .insert(Cow::Borrowed(&CODE_NAMESPACE), record.module().into());
        if !record.function().is_empty() {
            attrs
                .0
                .insert(Cow::Borrowed(&CODE_FUNCTION), record.function().into());
        }

        values.serialize(record, &mut attrs)?;
        record.kv().serialize(record, &mut attrs)?;

        let mut record_builder = LogRecord::builder()
            .with_context(&context)
            .with_timestamp((self.timestamp)())
            .with_severity_number(level_to_severity(record.level()))
            .with_severity_text(record.level().as_str())
            .with_attributes(attrs.0);

        record_builder = if let Some(body_str) = record.msg().as_str() {
            record_builder.with_body(body_str.into())
        } else {
            record_builder.with_body(record.msg().to_string().into())
        };

        if let Some(ref resource) = self.resource {
            record_builder = record_builder.with_resource(resource.clone());
        }

        self.emitter.emit(record_builder.build());
        Ok(())
    }
}

fn level_to_severity(level: Level) -> Severity {
    match level {
        Level::Critical => Severity::Fatal,
        Level::Error => Severity::Error,
        Level::Warning => Severity::Warn,
        Level::Info => Severity::Info,
        Level::Debug => Severity::Debug,
        Level::Trace => Severity::Trace,
    }
}

struct OtelSerializer(BTreeMap<Cow<'static, str>, Any>);

macro_rules! trivial_emit {
    ($name:ident, $ty: ty) => {
        fn $name(&mut self, key: Key, val: $ty) -> slog::Result {
            self.0.insert(key.into(), val.into());
            Ok(())
        }
    };
}

impl Serializer for OtelSerializer {
    fn emit_arguments(&mut self, key: Key, val: &Arguments<'_>) -> slog::Result {
        if let Some(val) = val.as_str() {
            self.0.insert(key.into(), Any::String(val.into()));
        } else {
            self.0.insert(key.into(), Any::String(val.to_string()));
        }
        Ok(())
    }

    fn emit_usize(&mut self, key: Key, val: usize) -> slog::Result {
        if val > std::i64::MAX as usize {
            self.emit_arguments(key, &format_args!("{}", val))
        } else {
            self.0.insert(key.into(), Any::Int(val as i64));
            Ok(())
        }
    }

    fn emit_isize(&mut self, key: Key, val: isize) -> slog::Result {
        if val > std::i64::MAX as isize {
            self.emit_arguments(key, &format_args!("{}", val))
        } else {
            self.0.insert(key.into(), Any::Int(val as i64));
            Ok(())
        }
    }

    fn emit_u128(&mut self, key: Key, val: u128) -> slog::Result {
        if val > std::i64::MAX as u128 {
            self.emit_arguments(key, &format_args!("{}", val))
        } else {
            self.0.insert(key.into(), Any::Int(val as i64));
            Ok(())
        }
    }

    fn emit_i128(&mut self, key: Key, val: i128) -> slog::Result {
        if val > std::i64::MAX as i128 {
            self.emit_arguments(key, &format_args!("{}", val))
        } else {
            self.0.insert(key.into(), Any::Int(val as i64));
            Ok(())
        }
    }

    #[cfg(feature = "nested-values")]
    fn emit_serde(&mut self, key: Key, value: &dyn SerdeValue) -> slog::Result {
        use serde_json::to_value;

        let value = to_value(value.as_serde()).map_err(|_| slog::Error::Other)?;
        self.0.insert(key.into(), value.into());
        Ok(())
    }

    trivial_emit!(emit_u8, u8);
    trivial_emit!(emit_i8, i8);
    trivial_emit!(emit_u16, u16);
    trivial_emit!(emit_i16, i16);
    trivial_emit!(emit_u32, u32);
    trivial_emit!(emit_i32, i32);
    trivial_emit!(emit_f64, f64);
    trivial_emit!(emit_f32, f32);
    trivial_emit!(emit_str, &str);
}
