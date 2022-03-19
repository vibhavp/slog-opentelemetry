OpenTelemetry Rust for slog-rs
==============================

# Warning

`slog-opentelemetry` is currently written against a forked version of
[`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust),
which adds support for logging. Till `v0.18.0` is not released, this
crate is not recommended for use in production, and is not currently
compatible with the [`tracing`](https://github.com/tokio-rs/tracing) crate.

# Usage

```toml
[dependencies]
slog = "2"
slog-opentelemetry = { git = "https://github.com/vibhavp/slog-opentelemetry" }
```
