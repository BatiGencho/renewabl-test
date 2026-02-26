#!/bin/bash
set -euo pipefail

# run in root directory

DATABASE_URL="postgres://username:password@localhost:5432/wire"

echo "Running diesel migrations..."
echo "Migration dir: ./db/migrations"
echo "Database URL: postgres://username:password@localhost:5432/wire"

# Run migrations
diesel migration run \
    --migration-dir ./db/migrations \
    --database-url "$DATABASE_URL"

echo "Migrations completed successfully."
