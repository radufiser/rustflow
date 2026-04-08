# Requirements Document

## Introduction

Section 3 adds full observability to RustFlow by replacing the existing `println!`-based logging with the `tracing` ecosystem. This covers structured logging, per-request spans, timing instrumentation, file-based log output, JSON-formatted logs, and distributed tracing via OpenTelemetry. The tracing infrastructure lives in `rustflow-common/src/tracing.rs` and is consumed by `rustflow-server`.

## Glossary

- **Tracing_Subscriber**: The global subscriber pipeline configured at application startup that collects, filters, and formats trace events and spans
- **EnvFilter**: A `tracing-subscriber` filter that controls log verbosity per module or crate using the `RUST_LOG` environment variable
- **TraceLayer**: A `tower-http` middleware layer that automatically creates spans and emits events for each HTTP request/response cycle
- **Span**: A period of time during which a program was executing in a particular context; spans can be nested and carry structured key-value fields
- **Event**: A moment in time within a span, representing a log message with structured fields
- **RustFlow_Server**: The Axum HTTP server crate (`rustflow-server`) that serves the REST API
- **RustFlow_Common**: The shared library crate (`rustflow-common`) that holds types, constants, and reusable infrastructure such as tracing setup
- **Tracing_Module**: The `rustflow-common/src/tracing.rs` module that provides tracing initialization functions
- **OTLP_Exporter**: An OpenTelemetry Protocol exporter that sends trace data to a collector (e.g., Jaeger)
- **Trace_Context**: The W3C `traceparent` / `tracestate` headers used to propagate distributed trace identity across service boundaries

## Requirements

### Requirement 1: Basic Tracing Initialization

**User Story:** As a developer, I want to initialize the `tracing` and `tracing-subscriber` crates in RustFlow, so that I can replace `println!` calls with structured trace events.

#### Acceptance Criteria

1. THE Tracing_Module SHALL export a function that initializes a global tracing subscriber with a `fmt` layer writing to stdout
2. WHEN RustFlow_Server starts, THE RustFlow_Server SHALL call the Tracing_Module initialization function before binding the TCP listener
3. WHEN the tracing subscriber is initialized, THE RustFlow_Server SHALL use `tracing::info!`, `tracing::debug!`, and `tracing::warn!` macros instead of `println!` for all log output

### Requirement 2: Log Level Filtering via EnvFilter

**User Story:** As a developer, I want to control log verbosity per module using the `RUST_LOG` environment variable, so that I can silence noisy dependencies and focus on RustFlow output.

#### Acceptance Criteria

1. THE Tracing_Module SHALL configure an EnvFilter that reads from the `RUST_LOG` environment variable
2. WHEN the `RUST_LOG` environment variable is not set, THE Tracing_Module SHALL apply a sensible default filter (e.g., `info` for RustFlow crates, `warn` for third-party crates)
3. WHEN the `RUST_LOG` environment variable is set, THE EnvFilter SHALL override the default and apply the user-specified directives

### Requirement 3: Automatic HTTP Request Logging via TraceLayer

**User Story:** As a developer, I want every HTTP request to RustFlow to be automatically logged with method, URI, and status code, so that I have visibility into all API traffic.

#### Acceptance Criteria

1. THE RustFlow_Server SHALL add a `tower-http` TraceLayer to the router layer stack
2. WHEN an HTTP request is received, THE TraceLayer SHALL create a span containing the HTTP method and request URI
3. WHEN an HTTP response is sent, THE TraceLayer SHALL emit an event containing the response status code and latency
4. THE TraceLayer SHALL replace the existing `log_request` and `log_elapsed_time` custom middleware functions

### Requirement 4: Timing Spans for Repository Operations

**User Story:** As a developer, I want spans around task, project, and user store operations, so that I can measure how long each data access takes.

#### Acceptance Criteria

1. WHEN a task store read or write operation begins, THE RustFlow_Server SHALL enter a span named with the operation (e.g., `task.list`, `task.create`)
2. WHEN a project or user store operation begins, THE RustFlow_Server SHALL enter a span named with the operation
3. THE Span SHALL include relevant fields such as the entity ID when available
4. WHEN the operation completes, THE Span SHALL automatically record its duration via the tracing subscriber

### Requirement 5: Custom Per-Request Axum Spans

**User Story:** As a developer, I want per-request spans enriched with domain-specific fields like task ID and authenticated user, so that I can correlate log output to specific requests.

#### Acceptance Criteria

1. THE RustFlow_Server SHALL configure the TraceLayer with a custom `make_span` function that creates a request-level span
2. THE request-level Span SHALL include the HTTP method, URI path, and a unique request ID
3. WHEN an authenticated client is present in request extensions, THE Span SHALL include the client name as a field
4. WHEN a path parameter contains an entity ID, THE Span SHALL include that ID as a field

### Requirement 6: Logging to a Rotated File

**User Story:** As a developer, I want RustFlow to write logs to a rotated file in addition to stdout, so that logs are persisted for production debugging.

#### Acceptance Criteria

1. THE Tracing_Module SHALL support an optional file logging layer that writes to a configurable log directory
2. WHEN file logging is enabled, THE Tracing_Module SHALL use `tracing-appender` to write logs to a file with daily rotation
3. WHEN file logging is enabled, THE Tracing_Module SHALL continue to emit logs to stdout simultaneously (dual output)
4. IF the log directory does not exist or is not writable, THEN THE Tracing_Module SHALL log a warning to stdout and continue without file logging

### Requirement 7: Structured JSON Log Output

**User Story:** As a developer, I want to switch RustFlow log output to JSON format, so that logs can be ingested by log aggregation systems (ELK, Datadog, etc.).

#### Acceptance Criteria

1. THE Tracing_Module SHALL support a JSON formatting mode for log output
2. WHEN JSON mode is enabled, THE Tracing_Module SHALL format each event as a single JSON object containing timestamp, level, target, message, and all span fields
3. WHEN JSON mode is disabled, THE Tracing_Module SHALL use the default human-readable `fmt` format
4. THE JSON formatting mode SHALL be selectable via a configuration parameter or environment variable

### Requirement 8: OpenTelemetry Trace Export

**User Story:** As a developer, I want to export RustFlow traces to an OpenTelemetry collector, so that I can visualize request flows in Jaeger or a similar tool.

#### Acceptance Criteria

1. THE Tracing_Module SHALL support an optional OpenTelemetry layer that exports spans via OTLP
2. WHEN OpenTelemetry export is enabled, THE OTLP_Exporter SHALL send trace data to a configurable collector endpoint
3. WHEN OpenTelemetry export is enabled, THE Tracing_Module SHALL set the service name to the RustFlow application name
4. WHEN OpenTelemetry export is disabled, THE Tracing_Module SHALL operate without any OpenTelemetry dependencies at runtime
5. IF the OTLP collector is unreachable, THEN THE Tracing_Module SHALL log a warning and continue operating without blocking request handling

### Requirement 9: Distributed Trace Context Propagation

**User Story:** As a developer, I want RustFlow to propagate W3C `traceparent` headers on outbound HTTP calls, so that traces span across service boundaries.

#### Acceptance Criteria

1. WHEN RustFlow_Server makes an outbound HTTP request (e.g., via the shared `http_client`), THE RustFlow_Server SHALL inject the current trace context as a `traceparent` header
2. WHEN RustFlow_Server receives an inbound HTTP request with a `traceparent` header, THE TraceLayer SHALL extract and continue the incoming trace context
3. THE trace context propagation SHALL use the W3C TraceContext propagator format
