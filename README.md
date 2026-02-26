# renewabl-test

A REST API for managing renewable energy plants, written in Rust.

## Features

- **CRUD operations** for renewable energy plants (solar, wind, hydro, geothermal, biomass, tidal)
- **Thread-safe in-memory store** using `Arc<RwLock<HashMap>>`
- **Async HTTP server** built with [Axum](https://github.com/tokio-rs/axum) and [Tokio](https://tokio.rs)
- **Structured logging** via `tracing`

## API Endpoints

| Method   | Path          | Description            |
|----------|---------------|------------------------|
| `GET`    | `/plants`     | List all plants        |
| `POST`   | `/plants`     | Create a new plant     |
| `GET`    | `/plants/:id` | Get a plant by ID      |
| `PUT`    | `/plants/:id` | Update a plant by ID   |
| `DELETE` | `/plants/:id` | Delete a plant by ID   |

## Data Model

```json
{
  "id": "uuid",
  "name": "Sunny Farm",
  "energy_type": "solar",
  "capacity_mw": 50.0,
  "location": "California, USA",
  "status": "active",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Energy Types

`solar` | `wind` | `hydro` | `geothermal` | `biomass` | `tidal`

### Plant Status

`active` | `inactive` | `maintenance`

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.75+

### Run the server

```bash
cargo run
```

The server listens on `http://0.0.0.0:3000` by default.

### Run the tests

```bash
cargo test
```

## Example Usage

```bash
# Create a solar plant
curl -s -X POST http://localhost:3000/plants \
  -H 'Content-Type: application/json' \
  -d '{"name":"Sunny Farm","energy_type":"solar","capacity_mw":50.0,"location":"California, USA"}'

# List all plants
curl -s http://localhost:3000/plants

# Update a plant
curl -s -X PUT http://localhost:3000/plants/<id> \
  -H 'Content-Type: application/json' \
  -d '{"status":"maintenance"}'

# Delete a plant
curl -s -X DELETE http://localhost:3000/plants/<id>
```
