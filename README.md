# rustflow

A task and workflow management service built in Rust.

This project is the companion codebase for the **Rust in Service-Oriented Architectures** course.

## Structure

| Crate | Purpose |
|---|---|
| `rustflow-server` | Main HTTP/WebSocket server |
| `rustflow-common` | Shared types, utilities, constants |

More crates will be added as the course progresses (`rustflow-db`, `rustflow-notifications`).

## Build 

```bash
# Build the entire workspace
cargo build
```
# Run the tests
```
cargo test --workspace
```
## Running

```bash
cargo run -p rustflow-server
```
```bash
# Root
curl -s http://localhost:3000/
# → RustFlow is running!

# Health (merged — no /api prefix)
curl -s http://localhost:3000/health | jq

# Config (merged — no /api prefix)
curl -s http://localhost:3000/config | jq

# Tasks (nested under /api)
curl -s http://localhost:3000/api/tasks | jq
curl -s http://localhost:3000/api/tasks/1 | jq
curl -s "http://localhost:3000/api/tasks?status=done" | jq
curl -s "http://localhost:3000/api/tasks?search=workspace" | jq

# Create a task
curl -s -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Test nested routing"}' | jq

# Update a task
curl -s -X PUT http://localhost:3000/api/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{"title": "Updated title", "priority": "low"}' | jq

# Change task status
curl -s -X PATCH http://localhost:3000/api/tasks/1/status \
  -H "Content-Type: application/json" \
  -d '"done"' | jq

# Delete a task
curl -i -X DELETE http://localhost:3000/api/tasks/3

# Projects (nested under /api)
curl -s http://localhost:3000/api/projects | jq

# Create a project
curl -s -X POST http://localhost:3000/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name": "New Project"}' | jq

# Old paths no longer work (404)
curl -i http://localhost:3000/tasks
# → 404 Not Found