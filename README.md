# rustflow

A task and workflow management service built in Rust.

This project is the companion codebase for the **Rust in Service-Oriented Architectures** course.

## Structure

| Crate | Purpose |
|---|---|
| `rustflow-server` | Main HTTP/WebSocket server |
| `rustflow-common` | Shared types, utilities, constants |

More crates will be added as the course progresses (`rustflow-db`, `rustflow-notifications`).

## Running

```bash
cargo run -p rustflow-server