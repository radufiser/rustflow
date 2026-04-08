# Implementation Plan: Section 3 — Tracing (Tutorial Documents)

## Overview

Create nine instructional markdown documents in `docs/sections/`, one per sub-section (3.1–3.9). Each document teaches the student what to implement and why, following the established format from Section 2 docs. These are tutorial guides — they describe changes, explain concepts, show code snippets as examples, but the student implements the code themselves.

All documents reference the tracing infrastructure described in the requirements and design: the `tracing` ecosystem replacing `println!`-based logging, with structured logging, filtering, HTTP request tracing, timing spans, file output, JSON formatting, OpenTelemetry export, and distributed trace propagation.

## Tasks

- [x] 1. Create section doc: `docs/sections/3.1-minimal-example.md`
  - Teach adding `tracing` and `tracing-subscriber` crates to the workspace
  - Explain creating `rustflow-common/src/tracing.rs` with a basic `init_tracing()` function that sets up a `fmt` subscriber writing to stdout
  - Show re-exporting the module from `rustflow-common/src/lib.rs`
  - Show calling `init_tracing()` in `rustflow-server/src/main.rs` before the TCP listener bind
  - Demonstrate replacing `println!` calls with `tracing::info!`, `tracing::debug!`, `tracing::warn!` macros
  - Include a File Changes table, step-by-step sections with code snippets, testing with `cargo run`, and exercises
  - Follow the format of existing section docs (Overview, File Changes, Steps, Test It, Project State, Exercises)
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. Create section doc: `docs/sections/3.2-log-levels-filtering.md`
  - Teach configuring `EnvFilter` from `tracing-subscriber` to control log verbosity per module
  - Explain reading from the `RUST_LOG` environment variable with a sensible compiled-in default (e.g., `rustflow_server=debug,rustflow_common=debug,tower_http=debug,warn`)
  - Show how `RUST_LOG` overrides the default when set
  - Demonstrate filtering noisy dependencies (e.g., `hyper=warn`, `tower=warn`)
  - Include examples of running with different `RUST_LOG` values and observing output changes
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 3. Create section doc: `docs/sections/3.3-logging-axum-tower.md`
  - Teach adding `tower-http`'s `TraceLayer` to the Axum router layer stack
  - Explain how `TraceLayer` automatically creates spans for each HTTP request (method, URI) and emits response events (status code, latency)
  - Show replacing the existing `log_request` and `log_elapsed_time` custom middleware functions with `TraceLayer`
  - Walk through the before/after of the middleware stack in `main.rs`
  - Include enabling the `trace` feature on `tower-http` in `Cargo.toml`
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4. Create section doc: `docs/sections/3.4-timing-spans.md`
  - Teach using `#[tracing::instrument]` to add named spans around store operations (task, project, user CRUD)
  - Explain span naming conventions (e.g., `task.list`, `task.create`, `project.list`, `user.create`)
  - Show including relevant fields like entity IDs in span attributes using `#[instrument(fields(task.id = %id))]`
  - Demonstrate how the tracing subscriber automatically records span duration
  - Walk through instrumenting handlers in `tasks.rs`, `projects.rs`, and `users.rs`
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 5. Create section doc: `docs/sections/3.5-axum-spans.md`
  - Teach configuring `TraceLayer` with a custom `make_span_with` function for per-request spans
  - Explain enriching request spans with: HTTP method, URI path, generated UUID request ID
  - Show including the authenticated client name from request extensions (`AuthenticatedClient`) when present
  - Show including entity IDs from path parameters when present
  - Walk through implementing a `MakeSpan` struct or closure and wiring it into the `TraceLayer`
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 6. Create section doc: `docs/sections/3.6-logging-to-file.md`
  - Teach adding `tracing-appender` for file-based log output with daily rotation
  - Explain the `NonBlocking` writer and the critical `WorkerGuard` lifetime pattern (guard must live for the application lifetime)
  - Show configuring dual output: logs go to both stdout and a rotated log file simultaneously
  - Explain graceful degradation: if the log directory doesn't exist or isn't writable, log a warning and continue without file logging
  - Show updating `TracingConfig` and `init_tracing()` to support the optional file layer
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 7. Create section doc: `docs/sections/3.7-structured-json-logging.md`
  - Teach switching the `fmt` layer to JSON output mode using `.json()` on the fmt layer
  - Explain the JSON event structure: timestamp, level, target, message, span fields
  - Show making JSON mode toggleable via a configuration parameter or environment variable
  - Demonstrate the difference between human-readable and JSON output with example log lines
  - Explain why JSON logs matter for log aggregation systems (ELK, Datadog, etc.)
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 8. Create section doc: `docs/sections/3.8-opentelemetry.md`
  - Teach adding optional OpenTelemetry support with `opentelemetry`, `opentelemetry-otlp`, `opentelemetry-sdk`, and `tracing-opentelemetry` crates
  - Explain feature-gating the OTel dependencies to avoid pulling them when not needed
  - Show configuring an OTLP exporter that sends spans to a collector endpoint (e.g., Jaeger)
  - Show setting the service name resource attribute to the RustFlow application name
  - Explain non-blocking export: if the collector is unreachable, the server continues operating with a warning
  - Include a brief guide on running Jaeger locally with Docker for testing
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [x] 9. Create section doc: `docs/sections/3.9-distributed-trace-propagation.md`
  - Teach W3C TraceContext propagation for distributed tracing across service boundaries
  - Explain injecting `traceparent` headers on outbound HTTP requests made via `state.http_client` (e.g., in `enrichment.rs`)
  - Explain extracting and continuing incoming `traceparent` headers on inbound requests via the `TraceLayer` or dedicated middleware
  - Show the `traceparent` header format: `{version}-{trace-id}-{parent-id}-{trace-flags}`
  - Walk through the enrichment endpoint as a concrete example of trace propagation across an outbound call
  - Tie together the full Section 3 progression and show the complete tracing pipeline
  - _Requirements: 9.1, 9.2, 9.3_

## Notes

- Each task produces exactly one markdown file in `docs/sections/`
- No Rust source files (.rs) or Cargo.toml files are created or modified
- Documents follow the established format from Section 2 docs: Overview, File Changes table, numbered Steps with code snippets, Test It section, Project State, Key Takeaways, Exercises
- Code snippets in the docs are illustrative examples for the student to follow — not auto-implemented code
- Each doc should reference the current state of the codebase (post Section 2.21) as its starting point
