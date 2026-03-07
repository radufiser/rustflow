# Rust in Service-Oriented Architectures

A comprehensive course covering REST server development, data handling, error management, modularization, tracing, OpenAPI documentation, configuration, gRPC, WebSockets, database integration, testing, containerized deployment, and service design — preparing you for deploying high-performance Rust services in enterprise environments.

## Course Project: RustFlow — A Task & Workflow Management Service

Throughout this course, you will build **RustFlow**, a task and workflow management service from the ground up. Each section adds new capabilities to the same codebase, mirroring how real-world services evolve. By the end, you will have a fully featured, production-ready service that includes:

- A REST API for managing tasks, projects, and users
- Real-time collaboration via WebSockets
- Internal service communication via gRPC (e.g., a notification microservice)
- Persistent storage with a relational database
- Full observability with structured logging, tracing, and OpenTelemetry
- Auto-generated OpenAPI documentation
- Environment-aware configuration and secrets management
- Containerized deployment with CI/CD
- Health checks, graceful shutdown, and scaling strategies

### Project Structure (evolves as you progress)

```text
rustflow/
├── Cargo.toml                  (workspace root)
├── config/
│   ├── default.toml
│   ├── development.toml
│   ├── production.toml
│   └── .env
├── proto/
│   └── notifications.proto     (added in Section 6)
├── migrations/                 (added in Section 8)
├── crates/
│   ├── rustflow-server/        (main REST + WS server)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── tasks.rs
│   │       │   ├── projects.rs
│   │       │   ├── users.rs
│   │       │   └── health.rs
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   ├── cors.rs
│   │       │   └── rate_limit.rs
│   │       ├── ws/             (added in Section 7)
│   │       │   ├── mod.rs
│   │       │   └── handler.rs
│   │       ├── extractors/
│   │       │   └── mod.rs
│   │       ├── errors/
│   │       │   └── mod.rs
│   │       ├── state.rs
│   │       └── config.rs
│   ├── rustflow-db/            (added in Section 8)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── task.rs
│   │       │   ├── project.rs
│   │       │   └── user.rs
│   │       └── repositories/
│   │           ├── mod.rs
│   │           ├── task_repo.rs
│   │           ├── project_repo.rs
│   │           └── user_repo.rs
│   ├── rustflow-notifications/ (added in Section 6)
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       ├── server.rs
│   │       └── client.rs
│   └── rustflow-common/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── tracing.rs      (added in Section 3)
│           └── config.rs       (added in Section 5)
├── tests/                      (added in Section 9)
│   ├── integration/
│   │   ├── api_tests.rs
│   │   └── grpc_tests.rs
│   └── load/
│       └── k6_script.js
├── Dockerfile                  (added in Section 10)
├── docker-compose.yml          (added in Section 10)
├── .github/
│   └── workflows/
│       └── ci.yml              (added in Section 10)
└── k8s/                        (added in Section 10)
    ├── deployment.yaml
    ├── service.yaml
    └── ingress.yaml
```

---

## Section 1: Introduction

- 1.1 - Course Overview & Objectives
- 1.2 - Prerequisites & Environment Setup
- 1.3 - Overview of Service-Oriented Architecture in Rust
- 1.4 - Project Introduction: RustFlow
    - Initialize the Cargo workspace
    - Create `rustflow-server` and `rustflow-common` crates
    - Verify the workspace builds and runs

---

## Section 2: REST Service

> **Project milestone:** Build the core REST API for RustFlow — task CRUD, routing, state management, authentication middleware, and error handling.

- 2.1 - Minimal HTTP Server
    - *Spin up a basic Axum server in `rustflow-server` that responds with "RustFlow is running"*
- 2.2 - Service Stack
    - *Understand the Axum/Tower/Hyper/Tokio stack powering RustFlow*
- 2.3 - Extractors
    - *Parse incoming task data using `Json`, `Path`, and `Query` extractors*
- 2.4 - Request Validation
    - *Validate task creation payloads (e.g., title length, due date format) using `validator` or `garde`*
- 2.5 - Add a Simple Tower Layer (State)
    - *Share an in-memory task list across handlers using `Arc<Vec<Task>>`*
- 2.6 - Add a Simple Tower Layer (Mutable State)
    - *Make the task list mutable with `Arc<RwLock<Vec<Task>>>` to support adding/updating tasks*
- 2.7 - Multiple States - Extension Layers
    - *Add separate state for projects and app configuration*
- 2.8 - Quick Recap on State and Layers
- 2.9 - Nesting Multiple Routers
    - *Create separate routers for `/api/tasks`, `/api/projects`, and `/api/users`*
- 2.10 - Nested Routers with State
    - *Ensure each sub-router has access to the correct shared state*
- 2.11 - Calling Other Services
    - *Add an endpoint that fetches external data (e.g., enriching a task with user info from a placeholder API)*
- 2.12 - Returning Status Codes
    - *Return `201 Created` for new tasks, `404 Not Found` for missing tasks, etc.*
- 2.13 - Using IntoResponse
    - *Create consistent API response wrappers for RustFlow*
- 2.14 - Error Handling with IntoResponse
    - *Build a unified `RustFlowError` type that maps to appropriate HTTP responses*
- 2.15 - Quick Recap on Nesting, Making Calls and Responses
- 2.16 - Serving Static Content with Tower
    - *Serve a simple RustFlow dashboard HTML page at `/`*
- 2.17 - Simple Header-Based Authentication
    - *Protect task mutation endpoints with an API key header*
- 2.18 - Simple Header-Based Auth with Middleware
    - *Refactor auth into a reusable middleware layer*
- 2.19 - Middleware Auth with Injection
    - *Inject the authenticated user identity into request extensions*
- 2.20 - Selectively Applying Layers
    - *Apply auth only to write endpoints; leave read endpoints open*
- 2.21 - CORS Configuration
    - *Configure CORS so the RustFlow dashboard (or external frontends) can call the API*
- 2.22 - Rate Limiting with Tower
    - *Add rate limiting to protect public-facing RustFlow endpoints*
- 2.23 - Router Layers
    - *Organize all middleware into a clean, composable layer stack*
- 2.24 - Graceful Shutdown
    - *Handle `SIGTERM`/`SIGINT` to drain in-flight requests before stopping RustFlow*
- 2.25 - Layer Recap

---

## Section 3: Tracing

> **Project milestone:** Add full observability to RustFlow — structured logs, request timing, and distributed tracing ready for OpenTelemetry.

- 3.1 - Minimal Example
    - *Add basic `tracing` and `tracing-subscriber` to RustFlow*
- 3.2 - Log Levels & Filtering
    - *Configure `EnvFilter` to control log verbosity per module (e.g., silence noisy dependencies)*
- 3.3 - Logging Axum/Tower
    - *Enable Tower's `TraceLayer` to automatically log every RustFlow request*
- 3.4 - Timing Spans
    - *Add spans around task repository operations to measure duration*
- 3.5 - Axum Spans
    - *Customize per-request spans with task IDs, user info, etc.*
- 3.6 - Logging to a File
    - *Write RustFlow logs to a rotated log file for production use*
- 3.7 - Structured Logging to JSON
    - *Switch to JSON log output for log aggregation systems (ELK, Datadog, etc.)*
- 3.8 - OpenTelemetry
    - *Export RustFlow traces to Jaeger or an OTLP collector*
- 3.9 - Distributed Trace Context Propagation
    - *Propagate `traceparent` headers when RustFlow calls other services or the notification microservice*

---

## Section 4: OpenAPI Documentation

> **Project milestone:** Auto-generate and serve interactive API docs for the entire RustFlow REST API.

- 4.1 - OpenAPI Basics & `utoipa` Integration
    - *Annotate RustFlow's task endpoints with `utoipa` macros*
- 4.2 - Documenting Request/Response Types
    - *Derive OpenAPI schemas for `Task`, `Project`, `User`, and error types*
- 4.3 - Serving Swagger UI / Redoc
    - *Serve interactive docs at `/docs` inside RustFlow*
- 4.4 - Generating Client SDKs from OpenAPI Specs
    - *Export the OpenAPI spec and generate a TypeScript client*
- 4.5 - Keeping Docs in Sync (CI Validation)
    - *Add a CI step that fails if the OpenAPI spec is out of date*

---

## Section 5: Handling Service Configuration

> **Project milestone:** Make RustFlow configurable across dev, staging, and production environments with secure secrets handling.

- 5.1 - Environment Variables with `.env`
    - *Load RustFlow's database URL, port, and log level from `.env`*
- 5.2 - The Config Crate - Basics
    - *Layer `default.toml` → `development.toml` → environment variables for RustFlow settings*
- 5.3 - Loading Config via HTTP
    - *Fetch RustFlow feature flags or config from a remote config service*
- 5.4 - CLI Configuration with Clap
    - *Add CLI flags to override RustFlow's port, config path, and log level*
- 5.5 - Secrets Management
    - *Use the `secrecy` crate for database passwords; discuss Vault / AWS Secrets Manager integration*
- 5.6 - Feature Flags for Service Configuration
    - *Toggle RustFlow features (e.g., WebSocket support, gRPC notifications) via configuration*
- 5.7 - Recap

---

## Section 6: gRPC

> **Project milestone:** Build a `rustflow-notifications` microservice that RustFlow calls via gRPC to send task assignment notifications.

- 6.1 - Hello Tonic - Protocol Definition
    - *Define `notifications.proto` with `NotifyTaskAssigned` and `NotifyTaskCompleted` RPCs*
- 6.2 - Hello Tonic - Project Definition and Build
    - *Set up the `rustflow-notifications` crate with `tonic-build`*
- 6.3 - Hello Tonic - The Server
    - *Implement the notification gRPC server*
- 6.4 - Hello Tonic - The Client
    - *Call the notification service from `rustflow-server` when a task is assigned*
- 6.5 - gRPC Streaming
    - *Introduce streaming concepts for batch notifications*
- 6.6 - gRPC Streaming - Protocol Definition
    - *Add a `StreamTaskUpdates` server-streaming RPC*
- 6.7 - gRPC Streaming - The Server
    - *Implement streaming task update notifications*
- 6.8 - gRPC Streaming - The Client
    - *Consume the stream from RustFlow or a CLI tool*
- 6.9 - Recap So Far
- 6.10 - Authentication
    - *Secure gRPC calls between RustFlow and the notification service with token-based auth*
- 6.11 - Tracing
    - *Propagate trace context across the gRPC boundary*
- 6.12 - gRPC Server Reflection
    - *Enable `tonic-reflection` for debugging with `grpcurl`*
- 6.13 - gRPC Health Checking Protocol
    - *Implement the standard health check service in `rustflow-notifications`*
- 6.14 - Running gRPC + REST on the Same Service
    - *Expose both REST and gRPC on `rustflow-server` using `tonic-web`*
- 6.15 - When to Use gRPC

---

## Section 7: WebSockets

> **Project milestone:** Add real-time collaboration to RustFlow — users see task updates live without refreshing.

- 7.1 - Minimal Echo Server
    - *Add a `/ws` endpoint to RustFlow that echoes messages*
- 7.2 - A Native WS Client
    - *Build a simple Rust CLI client that connects to RustFlow's WebSocket*
- 7.3 - JSON
    - *Send and receive structured `TaskUpdate` JSON messages over the WebSocket*
- 7.4 - Connection Management & Heartbeats
    - *Implement ping/pong, detect stale connections, handle reconnection*
- 7.5 - Broadcasting / Pub-Sub Pattern
    - *Use `tokio::sync::broadcast` to push task changes to all connected clients in real time*

---

## Section 8: Data Layer

> **Project milestone:** Replace in-memory storage with a real database. RustFlow tasks, projects, and users are now persisted.

- 8.1 - Choosing a Database Strategy
    - *Why PostgreSQL for RustFlow; brief comparison with other options*
- 8.2 - Connection Pooling with `sqlx` + `deadpool`
    - *Set up a connection pool in `rustflow-db` and inject it into Axum state*
- 8.3 - Database Migrations
    - *Create migration files for `tasks`, `projects`, and `users` tables*
- 8.4 - The Repository Pattern for Data Access
    - *Implement `TaskRepository`, `ProjectRepository`, and `UserRepository` in `rustflow-db`*
- 8.5 - Integrating the Data Layer into Axum Services
    - *Swap out in-memory state for database-backed repositories in `rustflow-server` handlers*

---

## Section 9: Testing

> **Project milestone:** Ensure RustFlow is reliable with unit, integration, contract, and load tests.

- 9.1 - Unit Testing Service Logic
    - *Test task validation, error mapping, and business rules in isolation*
- 9.2 - Integration Testing Axum Services
    - *Spin up a test `rustflow-server` instance and run HTTP tests with `reqwest`*
- 9.3 - Contract Testing
    - *Validate that RustFlow's API matches its OpenAPI spec*
- 9.4 - Load Testing Basics
    - *Use k6 or similar to load test RustFlow's task creation endpoint*
- 9.5 - Testing Recap

---

## Section 10: Service Deployment

> **Project milestone:** Package and deploy RustFlow as a production-ready containerized service.

- 10.1 - Test Service
    - *Final smoke test of the complete RustFlow feature set*
- 10.2 - Health Checks & Readiness Probes
    - *Implement `/health` and `/ready` endpoints that verify DB connectivity and service state*
- 10.3 - Native Host Deployment
    - *Build a release binary and run RustFlow directly on a host*
- 10.4 - Docker Deployment
    - *Write a multi-stage Dockerfile; produce a minimal image using `scratch` or `distroless`*
- 10.5 - CI/CD Pipeline
    - *Set up a GitHub Actions workflow: lint → test → build → push Docker image*
- 10.6 - Kubernetes Deployment
    - *Write Deployment, Service, and Ingress manifests; introduce Helm basics*

---

## Section 11: Service Design

> **Project milestone:** Reflect on RustFlow's architecture and learn how to evolve, expose, and scale it in an enterprise setting.

- 11.1 - Understanding Your Company Architecture
    - *Map RustFlow into typical enterprise patterns (API gateway, service mesh, event bus)*
- 11.2 - Designing Individual Services
    - *Review RustFlow's internal design: separation of concerns, dependency injection, error boundaries*
- 11.3 - Combining Services into a Modular Monolith
    - *Discuss keeping `rustflow-server` and `rustflow-notifications` in one deployable unit vs. splitting*
- 11.4 - Observability-Driven Design
    - *Tie together RustFlow's tracing, structured logging, and metrics into a unified observability strategy*
- 11.5 - Service Exposure
    - *API gateways, load balancers, and public vs. internal API surfaces for RustFlow*
- 11.6 - Scaling Out
    - *Horizontal scaling strategies, stateless design, database connection management at scale*

---

## Appendix

- A.1 - Recommended Crates Reference
- A.2 - Further Reading & Resources
- A.3 - Troubleshooting Common Issues
- A.4 - Complete RustFlow API Reference