# Energy Readings API (also called wire-api)

A Rust backend service for querying timeseries energy data. Built with Axum, Diesel (PostgreSQL), and Redis caching. Reads hourly energy readings from an Excel file, loads them into Postgres on startup, and exposes aggregation endpoints (hourly, daily, monthly) with date filtering and query history.

## Prerequisites

- Rust 1.92+ (see `rust-toolchain`)
- Docker & Docker Compose
- [Tilt](https://tilt.dev/) for local orchestration
- bun.js (for pre-commit hooks, formatting, and linting)
- `cargo-nextest` for running tests
- `taplo` for TOML formatting
- `cargo-machete` for unused dependency checks

## Getting Started

### 1. Tooling Setup

There's a setup script that installs all the cargo tools you'll need (nextest, taplo, cargo-machete, cargo-watch, cargo-audit, bacon, etc.):

```bash
./scripts/setup.sh
```

It also installs bun (if missing), runs `bun install`, and sets up pre-commit hooks. If you'd rather install things yourself, check the script to see the full list.

### 2. Environment Setup

Copy the sample env file and fill in any values you need to change:

```bash
cp .env.sample .env
```

The defaults should work fine for local development. Make sure `ENERGY_READINGS_XLS_FILE_PATH` points to the Excel file (the test data file is at the project root: `Test January2025-December2025-hourly-example.xlsx`).

### 3. Install Dependencies

```bash
bun install
```

This pulls in husky, commitlint, prettier, markdownlint, and other dev tooling. It also sets up the pre-commit hooks via husky, so your commits will be automatically linted before they go through.

### 4. Running the Project

There are two ways to run everything locally:

**Option A: Infrastructure only (run the API on your machine)**

This spins up just Postgres, Redis, and the monitoring stack -- useful when you want to iterate on the Rust code without rebuilding a Docker image every time.

```bash
tilt up --file Tiltfile local-api-testing
```

Then in another terminal:

```bash
cargo run --bin wire-api
```

**Option B: Full stack in Docker**

This builds and runs the API container alongside all infrastructure:

```bash
tilt up
```

The API will be available on the port defined in your `.env` (`API_SERVICE_PORT`, default `50051`).

### 5. Swagger / OpenAPI

Once the API is running:

- Swagger UI: <http://localhost:50051/swagger-ui>
- OpenAPI spec: <http://localhost:50051/api-docs/openapi.json>

## Testing

The test data file used for this project lives at the repo root:

```
Test January2025-December2025-hourly-example.xlsx
```

It contains hourly energy readings for the full year of 2025 with `Time (UTC)` and `Quantity kWh` columns.

There is also a Postman collection inside `postman/` that you can import to quickly test the endpoints manually.

### Running Tests

```bash
make test            # run all workspace tests
make test-verbose    # run with full status output
make test-package PKG=excel_client   # run tests for a specific crate
```

## Formatting & Linting

The Makefile has everything you need:

```bash
make fmt       # format everything (Rust, TOML, Prettier, Markdown)
make lint      # lint everything (cargo check, clippy, prettier, markdown, machete)
```

You can also run individual targets:

```bash
make fmt-rust       # cargo fmt
make fmt-cargo      # taplo (TOML formatting)
make lint-clippy    # clippy with -D warnings
make lint-machete   # check for unused dependencies
```

## Other Useful Make Targets

```bash
make build          # release build
make clean          # clean target/ and node_modules/
make dev-watch      # cargo watch for auto-reload during development
make audit          # cargo audit for security vulnerabilities
make docs           # generate and open Rust docs
make docs-serve     # serve docs on localhost:8000
```

## API Endpoints

- `POST /api/wire/v1/energy/aggregate` -- query energy data with aggregation (hourly, day_of_month, monthly) and optional date filters
- `GET /api/wire/v1/energy/history` -- retrieve the last 10 queries with their filter values

## How It Works

On startup the API reads the Excel file and bulk-inserts the readings into the `energy_readings` table (idempotent -- skips if data already exists). Aggregation queries run against a read-only connection pool and results are cached in Redis to keep things snappy under concurrent load.
