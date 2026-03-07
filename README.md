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