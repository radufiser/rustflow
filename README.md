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

# List initial tasks (3 seed tasks)
curl -s http://localhost:3000/tasks | jq '. | length'
# → 3

# Create a new task — now it persists in state!
curl -s -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "New task from 2.6!"}' | jq

# List again — should be 4
curl -s http://localhost:3000/tasks | jq '. | length'
# → 4

# Create another
curl -s -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Another task", "priority": "high"}' | jq
# → id: 5

# Delete a task
curl -i -X DELETE http://localhost:3000/tasks/1
# → 204 No Content

# Verify it's gone
curl -i http://localhost:3000/tasks/1
# → 404 Not Found

# Delete non-existent (404)
curl -i -X DELETE http://localhost:3000/tasks/999
# → 404 Not Found
```