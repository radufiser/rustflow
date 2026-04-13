# AGENTS.md

AI-assisted development guide for RustFlow — a Rust-based task and workflow management service.

## Purpose

This is a **learning project** driven by a structured course (`/docs/Course.md`).
The goal is to become proficient at building **low-latency, high-throughput services in Rust** for enterprise environments — covering REST, gRPC, WebSockets, database integration, observability, configuration, containerized deployment, and scaling.

**Section docs** (`/docs/sections/`) are instructional guides: they describe *what* to change and *why*, but the student implements the code themselves. AI agents working in this repo should **never auto-implement a section's changes** — instead, help explain, debug, or extend what the student writes.

## Architecture Overview

Cargo workspace with two crates (more planned per `Course.md`):

- **`rustflow-server`** — Axum HTTP server (routes, middleware, state, errors, extractors)
- **`rustflow-common`** — Shared types, validation rules, constants (`AuthenticatedClient`, `Task`, `CreateTask`, etc.)
- **Future**: `rustflow-db` (Section 8), `rustflow-notifications` gRPC (Section 6)

### Key Decisions

1. **Router layout**: Domain routes nested under `/api/*`, infrastructure merged at root
   - `GET /health` (merged) vs `GET /api/tasks/1` (nested)
2. **Composite state**: `AppState` holds `TaskState`, `ProjectState`, `UserState` (each `Arc<RwLock<Store>>`), plus shared `http_client`, `config`, `api_keys`, `request_counter: Arc<AtomicU64>`, and `shutdown_requested: Arc<AtomicBool>`
3. **Unified errors**: `RustFlowError` enum → `IntoResponse` → structured JSON with `request_id`
4. **Custom extractors**: `ValidatedJson<T>` / `ValidatedQuery<T>` combine deserialization + validation → 422 on failure
5. **Public/protected split** (since 2.17): each domain router splits reads (open) from writes (auth required via `require_api_key` middleware)

## Build & Run

```bash
cargo build                       # workspace
cargo run -p rustflow-server      # server on :3000
cargo test --workspace            # unit + doc tests
```

```bash
# Read (open)
curl http://localhost:3000/api/tasks | jq

# Write (requires x-api-key header — keys in config/api_keys.json)
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -H "x-api-key: rustflow-api-key-2026" \
  -d '{"title": "Test task"}' | jq
```

## Critical Conventions

### Types vs Create Types
`Task` (has `id`, `status`) vs `CreateTask` (client provides title/description/priority only). Create types carry `#[validate]` attrs.

### Lock Scope Pattern (CRITICAL)
```rust
// ✅ Lock scoped, dropped before I/O
let task = {
    let store = state.tasks.0.read().await;
    store.tasks.iter().find(|t| t.id == id).cloned()
};
let external_data = http_client.get(url).await?;
```

### Router Factory — Public/Protected Split
Each module exports `router(state) -> Router<AppState>`:
```rust
pub fn router(state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/", get(list))
        .route("/{id}", get(get_one));

    let protected = Router::new()
        .route("/", post(create))
        .route("/{id}", put(update).delete(delete))
        .layer(middleware::from_fn_with_state(state, require_api_key));

    public.merge(protected)
}
```

### Workspace Dependencies
New crates inherit versions from workspace root `Cargo.toml`:
```toml
axum = { workspace = true }
```

### Edition 2024
All crates use `edition = "2024"`.

### Tracing Instrumentation Pattern
Handlers use `#[tracing::instrument]` with domain-scoped span names and selective field recording:
```rust
#[tracing::instrument(name = "task.create", skip_all, fields(task.id))]
async fn create(...) -> Result<...> {
    // ... do work ...
    tracing::Span::current().record("task.id", task.id); // late-record once known
    tracing::info!(task_id = %task.id, client = %client.name, "task created");
}
```
- `name = "domain.action"` — e.g. `task.list`, `enrichment.enrich`
- `skip_all` — avoid logging large state; select fields explicitly
- Sub-spans for I/O: `.instrument(tracing::info_span!("http.get", url = %url))` (see `enrichment.rs`)

## Course-Driven Development

Current state: **Section 3.5** (tracing with named Axum spans and `#[instrument]` on handlers).

**Next section** (doc exists in `/docs/sections/`, ready to implement):
- 3.6 — Logging to File

**Patterns established so far:**
- Auth middleware with identity injection (`middleware.rs` → `AuthenticatedClient` in extensions)
- Public/protected router splitting (reads open, writes require API key)
- API keys loaded from `config/api_keys.json` at startup
- CORS configured from `AppConfig.cors_origins` (dynamic origin list)
- Per-router rate limiting via `rate_limited()` helper (see below)
- Router groups by middleware scope: static (none), health (merged at root), API (CORS + compression + request counter)
- Chained `.layer()` ordering: **last applied = outermost** (runs first on request)
- Graceful shutdown: `Notify` + `tokio::select!` + 10s drain timeout (`main.rs`)
- Tracing: `init_tracing()` in `rustflow-common`, `TraceLayer` for HTTP, `#[tracing::instrument]` on handlers
- Response compression: `CompressionLayer` with `SizeAbove(512)` on API router
- Request counter middleware: `request_counter` logs every 100th request

### Tower Layer Stack Pattern (CRITICAL)
Bare `RateLimitLayer` / `LoadShedLayer` won't compile with Axum — two fixes required:
- `BufferLayer` → makes `RateLimit` cloneable (Axum's Hyper clones the service per TCP connection)
- `HandleErrorLayer` → converts `BoxError` to HTTP response (`LoadShed` produces `BoxError`, Axum expects `Infallible`)

This is encapsulated in the `rate_limited()` helper (`middleware.rs`), which also rewrites `503 → 429` and injects a `Retry-After` header:
```rust
// Per-router rate limiting with different limits for reads vs writes
let public = rate_limited(Router::new().route("/", get(list)), 100, Duration::from_secs(10));
let protected = rate_limited(Router::new().route("/", post(create)).layer(...), 10, Duration::from_secs(10));
```

**Planned architecture** (not yet implemented):
- Section 6: gRPC notifications (`rustflow-notifications` crate)
- Section 7: WebSocket real-time collaboration
- Section 8: Database layer (`rustflow-db` with repository pattern)

## Integration Points

- **External HTTP**: shared `state.http_client` (reqwest) — see `enrichment.rs` for lock-scoping + external call pattern
- **Static content**: `tower-http` `ServeFile`/`ServeDir` — paths relative to workspace root
- **Tests**: unit tests in `rustflow-common/src/lib.rs`; integration tests planned for Section 9

---
inclusion: always
---

# Shell Command Guidelines

CRITICAL: Do not use interactive commands or pagers (e.g., less, more, vim). All shell commands must be non-interactive. 
Ensure you use flags like -y for confirmations and --no-pager for CLI tools. 
If a command typically opens a pager, pipe it to cat or set PAGER=cat in the environment to ensure the full output
is returned without waiting for user input.