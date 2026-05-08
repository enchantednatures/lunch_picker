# Architecture Overview

## Project Identity

`lunch_picker` is a Rust CLI application that helps groups ("homies") decide where to eat for lunch. It manages a local SQLite database of people, restaurants, favorites, and recent picks, then uses a weighted random algorithm to suggest restaurants while excluding recently visited ones.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (Edition 2021) |
| Runtime | Tokio (rt-multi-thread) |
| Database | SQLite via `sqlx` |
| CLI Parsing | `clap` (derive macros) |
| Interactive Prompts | `dialoguer` |
| Observability | `tracing` + OpenTelemetry OTLP |
| Serialization | `serde` + `serde_json` |
| Error Handling | `anyhow` + `thiserror` |

## Directory Structure

```
shitty_lunch_picker/
├── src/
│   ├── main.rs              # CLI entry point, app lifecycle, telemetry init
│   ├── lib.rs               # Module declarations, public re-exports
│   ├── cli_args.rs          # Clap-derived argument and subcommand enums
│   ├── config.rs            # Settings, DatabaseSettings, SqliteSettings
│   ├── db.rs                # Migrator trait + sqlx migration runner
│   ├── interaction.rs       # Interactive terminal UI (dialoguer prompts)
│   ├── user.rs              # UserId newtype wrapper
│   └── features/            # Domain feature modules
│       ├── homies/          # CRUD for homies
│       ├── restaurants/     # CRUD for restaurants + candidate selection
│       ├── recents/         # Recording recent restaurant picks
│       ├── homies_favorites/# Linking homies to favorite restaurants
│       ├── recipes/         # Dead code — partially implemented
│       ├── get_homie_by_name.rs # Incomplete — no trait impl
│       └── ...empty modules (read_homie, update_homie, etc.)
├── tests/                   # Integration tests (requires `sqlite_tests` feature)
├── migrations/              # SQLx migration files
├── .sqlx/                   # Compiled query metadata for offline builds
├── .github/workflows/       # CI/CD (build, test, clippy, fmt, release)
└── config.toml              # Cargo build config (aarch64-apple-darwin target)
```

## Application Lifecycle

```
main()
  ├── Parse CLI args (clap)
  ├── Load/create config at ~/.config/local/lunch.json
  ├── Initialize OpenTelemetry tracer (if enabled)
  ├── Ensure SQLite DB exists
  ├── Connect with SqlitePoolOptions (max_connections: 5)
  ├── Run migrations (sqlx::migrate!)
  ├── Instantiate AppState { db }
  └── Dispatch to command handler or default interactive flow
```

## Core Data Flows

### 1. Default Interactive Flow (`AppState::work`)

Triggered when no subcommand is given:

1. Fetch all homies for user. If empty → interactive homie + restaurant setup.
2. Prompt user to select which homies are "home" today (`get_home_homies`).
3. Query candidate restaurants (`get_candidate_restaurants`) — restaurants that are favorites of ALL home homies, excluding recent picks.
4. If no candidates → prompt to add more restaurants and retry.
5. Present candidate list for user selection (`select_restaurant`).
6. Record selection as recent for all home homies (`add_recent_restaurant_for_homies`).

### 2. CLI Subcommand Flows

- `homies add <name>` → `create_homie`
- `homies restaurants add <homie> <restaurant>` → `add_homies_favorite_restaurant`
- `homies restaurants delete <homie> <restaurant>` → `remove_homies_favorite_restaurant`
- `restaurants add <name>` → `create_restaurant`
- `pick` → Same as default interactive flow

## Module Architecture

### Feature Module Pattern

Each feature follows a consistent (though repetitive) structure:

```
features/<feature>/
  ├── models.rs          # Domain types, newtype wrappers, validation
  ├── <action>.rs        # Public async fn + private Params struct + trait + impl
  └── ...
```

Example: `features/homies/create.rs`
- Public function: `create_homie(name, user_id, db) -> Result<Homie, CreateHomieError>`
- Private params struct: `CreateHomieParams<'a>`
- Repository trait: `CreateHomie` with async method
- Trait implementation: `impl CreateHomie for Pool<Sqlite>`

### Trait-Based Repository Pattern

The codebase uses a trait-based repository pattern where each DB operation defines a trait:

```rust
pub trait CreateHomie {
    async fn create_homie(&self, params: CreateHomieParams<'_>) -> Result<Homie, CreateHomieError>;
}

impl CreateHomie for Pool<Sqlite> { ... }
```

**Intent**: Decouple business logic from DB implementation, enable testability with mock DBs.

**Reality**: Every trait is implemented *only* for `Pool<Sqlite>`. No mock implementations exist. The trait bounds in interactive functions create significant boilerplate with no payoff.

### Newtype Pattern

Strong typing for IDs and names:
- `HomieId(i32)`, `RestaurantId(i32)`, `UserId(i32)`
- `HomiesName(String)`, `RestaurantName(String)`

These provide compile-time safety against mixing IDs and include `TryFrom<String>` validation for names (non-empty, trimmed).

## Configuration

- **Config file**: `~/.config/local/lunch.json` (JSON)
- **Default DB**: `~/.local/state/lunch.db` (SQLite)
- **Env override**: `DATABASE_URL` overrides config setting
- **Telemetry**: Optional OTLP exporter to `http://localhost:4317`

## Database Layer

- **Driver**: `sqlx` with SQLite, `runtime-tokio-rustls`
- **Migrations**: Embedded via `sqlx::migrate!("./migrations")`
- **Connection pool**: Max 5 connections
- **Query checking**: Offline `.sqlx/` JSON files enable compile-time query verification without a running DB
- **Macros used**: `query_as!` for typed rows, `query!` for dynamic columns

## Observability

- `tracing` spans instrument DB queries and user interactions
- OpenTelemetry pipeline exports to local collector (4317)
- Global subscriber set when `telemetry_enabled: true`
- AppState's `Drop` impl shuts down tracer provider and closes DB pool

## Build & Release

- **Profiles**: Release optimized for size (`opt-level = 's'`, `lto = true`, `panic = 'abort'`)
- **CI**: GitHub Actions — build/test on Ubuntu + macOS, format check, clippy
- **Release**: Automated binary upload on tag push (`taiki-e/upload-rust-binary-action`)
- **Target**: Config defaults to `aarch64-apple-darwin` (M-series Macs)
