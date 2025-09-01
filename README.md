# Hexagonal/Clean Architecture Overview

This project follows a **Hexagonal/Clean Architecture** with a clear composition root and one–way dependencies.

```
src/
 ├─ main.rs            # entrypoint (runtime & serve)
 ├─ bootstrap.rs       # composition root (wiring)
 ├─ api/               # delivery/HTTP layer (Axum)
 ├─ application/       # use-cases (orchestration)
 ├─ domain/            # core business + ports (traits)
 └─ infrastructure/    # adapters (DB, cache, clients, etc.)
```

## Responsibilities

### `main.rs` — Entrypoint
- Starts the runtime (logging/tracing/Tokio).
- Binds the TCP listener and calls `axum::serve`.
- Contains no business logic or wiring details.

### `bootstrap.rs` — Composition Root
- Builds concrete adapters (e.g., Postgres vs. in-memory), services, and the HTTP router/state.
- The only module allowed to import **everything** (api + application + domain + infra).
- Great place to switch implementations by config/env/feature flags.
- Provide helpers for tests (e.g., `build_test_router()` using in-memory adapters).

### `api/` — Delivery/HTTP Layer
- Axum routes and handlers (HTTP ⇄ DTOs).
- Handlers pull only the dependencies they need via `State<…>` (often with `FromRef`).
- Maps domain/application errors into HTTP status codes.
- No database or business rules here.

### `application/` — Use‑Cases
- Orchestrates flows across domain objects and ports (e.g., `CreateUser`, `GetUser`).
- Depends on **domain ports (traits)** only; unaware of specific databases/HTTP.
- A natural place for transaction boundaries and cross-entity workflows.

### `domain/` — Core
- Entities, value objects, domain services, and domain errors.
- **Ports (traits)** such as `UserRepository` are defined here.
- Pure business rules; framework‑agnostic; minimal dependencies.

### `infrastructure/` — Adapters
- Implements domain ports: DB repositories (sqlx/SeaORM), caches (Redis), MQ, external HTTP clients, logging, config, migrations.
- Swappable without touching `domain`/`application` code.

## Dependency Direction

```
api  ─▶  application  ─▶  domain
                       ▲
                       └── infrastructure (implements domain ports)
bootstrap depends on all to assemble the app
```

- **Only the composition root** (`bootstrap.rs`) wires concrete implementations together.
- **api** depends on **application**, **application** depends on **domain** (ports), and **infrastructure** implements those ports.

## Typical Request Flow

`POST /users` → `api` handler → `application` use‑case → domain validation → call `UserRepository` (port) → `infrastructure` repository hits DB → result mapped back to HTTP.

## Production Notes

- **Testability:** domain/application can be unit‑tested with mocks; API/infra with integration tests.
- **Swappable adapters:** change DB/clients in `infrastructure/`, adjust wiring only in `bootstrap.rs`.
- **Clear boundaries:** each layer has a single responsibility; accidental cross‑layer imports are easy to spot.
- **Performance option:** keep services **concrete** (no `dyn`) and use **closure handlers** in `api` so the compiler **monomorphizes** them (no vtable on hot paths). Use `dyn` only when you need maximum runtime flexibility.
- **State cloning:** prefer storing types that are cheap to clone (e.g., `sqlx::Pool`, `reqwest::Client` are already cheap/Arc-backed). Wrap custom non‑clone resources in `Arc<T>` only when necessary.

## Getting Started (example sketch)

```rust
// bootstrap.rs
use std::sync::Arc;
use crate::{api, application::UserServiceImpl, infrastructure::in_memory_repo::InMemoryUserRepository};

pub fn build_router() -> axum::Router {
    let repo = Arc::new(InMemoryUserRepository::new());
    let user_svc = Arc::new(UserServiceImpl::new(repo));

    let services = api::Services { user: user_svc };
    api::router(services) // returns Router<()> — ready to serve
}
```

```rust
// main.rs
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = crate::bootstrap::build_router();
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Tips

- Use `FromRef` (or small newtype wrappers + `FromRef`) so each handler requests only the dependency it needs.
- Put environment/config parsing in `bootstrap.rs`; keep `api/application/domain` framework‑agnostic.
- For multiple binaries (API, worker, migrator), create `src/bin/<name>.rs` and share `bootstrap` helpers.
