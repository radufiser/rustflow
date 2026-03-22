# AGENTS.md

AI-assisted development guide for RustFlow — a Rust-based task and workflow management service.

## Architecture Overview

RustFlow is a **Cargo workspace** with an evolving multi-crate structure following a course-driven development model:

- **`rustflow-server`**: Main HTTP/WebSocket server using Axum
- **`rustflow-common`**: Shared types, validation rules, and constants
- **Future crates** (per `/docs/Course.md`): `rustflow-db`, `rustflow-notifications` (gRPC), as the project evolves through Sections 6-11

### Key Architectural Decisions

1. **Router Organization**: Domain endpoints are nested under `/api/*` (tasks, projects, users), while infrastructure endpoints (health, config) are merged at root level with no prefix
   - Example: `GET /health` vs `GET /api/tasks/1`
   - See `main.rs` `.merge()` vs `.nest()` pattern

2. **State Management**: Uses a **composite state pattern** with independently lockable stores
   - `AppState` contains `TaskState`, `ProjectState`, `UserState` (each with `Arc<RwLock<Store>>`)
   - Lock scoping is critical: always drop locks before I/O operations (see `enrichment.rs` pattern)
   - Shared `http_client` (reqwest) and immutable `config` (AppConfig) live at AppState level

3. **Error Handling**: Unified `RustFlowError` enum (in `errors.rs`) with `IntoResponse` impl
   - Each variant maps to specific HTTP status + structured JSON response with `request_id`
   - Handlers return `Result<T, RustFlowError>` — no bare `StatusCode` returns

4. **Validation**: Custom extractors (`ValidatedJson`, `ValidatedQuery`) combine deserialization + validation
   - Rules defined in `rustflow-common` types via `#[validate(...)]` attributes
   - Returns 422 Unprocessable Entity with field-specific error messages on failure

## Development Workflows

### Build & Run
```bash
# Build entire workspace
cargo build

# Run the server (default port 3000)
cargo run -p rustflow-server

# Run tests (includes doc tests in rustflow-common)
cargo test --workspace
```

### Testing API Endpoints
```bash
# Infrastructure (merged at root)
curl http://localhost:3000/health | jq
curl http://localhost:3000/config | jq

# Domain endpoints (nested under /api)
curl http://localhost:3000/api/tasks | jq
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Test task"}' | jq

# Note: Old paths like /tasks (without /api) return 404 by design
```

## Critical Conventions

### 1. Types vs Create Types
Separate domain types from creation payloads:
- `Task` (has `id`, `status`) vs `CreateTask` (no `id`, client doesn't set status)
- `Project` vs `CreateProject`, `User` vs `CreateUser`
- Pattern: Create types have `#[validate]` attrs, domain types don't

### 2. Lock Scope Pattern (CRITICAL)
When combining state access + external I/O:
```rust
// ✅ CORRECT: Lock scoped in block, dropped before HTTP call
let task = {
    let store = state.tasks.0.read().await;
    store.tasks.iter().find(|t| t.id == id).cloned()
}; // <- lock dropped here
let external_data = http_client.get(url).await?;
```
```rust
// ❌ WRONG: Lock held during I/O, blocks all handlers
let store = state.tasks.0.read().await;
let task = store.tasks.iter().find(|t| t.id == id);
let external_data = http_client.get(url).await?; // lock still held!
```

### 3. Router Factory Pattern
Each module exports a typed `router()` function:
```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete))
}
```
Main composes these with `.nest()` or `.merge()` + `.with_state(state)` at the end.

### 4. Workspace Dependencies
New crates inherit shared versions from workspace root `Cargo.toml`:
```toml
# In crate Cargo.toml
axum = { workspace = true }
tokio = { workspace = true }
```
Always check workspace root first when adding dependencies.

### 5. Edition 2024
This project uses Rust edition 2024 — ensure new crates specify `edition = "2024"` in Cargo.toml.

## Course-Driven Development

This codebase evolves alongside `/docs/Course.md`. Current state: **Section 2.13** (serving static content completed).

**When implementing new sections:**
1. Check `/docs/sections/` for lesson-specific implementation notes
2. Cross-reference `Course.md` structure to understand where new components fit
3. Upcoming milestones add: authentication (2.14-2.17), CORS (2.18), rate limiting (2.19), graceful shutdown (2.21)

**Planned architecture changes** (not yet implemented):
- Section 6: gRPC notifications service (new crate `rustflow-notifications`)
- Section 7: WebSocket real-time collaboration (`/ws` endpoint)
- Section 8: Database layer (new crate `rustflow-db` with repository pattern)

## Integration Points

### External Service Calls
Use shared `state.http_client` (reqwest::Client) for all external HTTP:
- Lives in `AppState` (reuses connection pools)
- Error mapping: `reqwest::Error -> RustFlowError::ExternalService`
- Example: `enrichment.rs` fetches from JSONPlaceholder API

### Static Content
Served via `tower-http::services`:
- Root `/` → single file (`ServeFile::new("crates/rustflow-server/static/index.html")`)
- `/static/*` → directory (`ServeDir::new(...).append_index_html_on_directories(true)`)
- Paths are relative to workspace root (where `cargo run` executes)

## Testing Patterns

Unit tests in `rustflow-common/src/lib.rs` verify:
- Default values on domain types (HealthStatus, Priority, TaskStatus)
- Serde deserialization with defaults (`#[serde(default)]`)
- Validation rules (not yet comprehensive)

**Note**: Integration tests not yet implemented (planned for Section 9).

