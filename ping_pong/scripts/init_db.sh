#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
    echo "Error: psql is not installed." >&2
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed." >&2
    echo >&2 "Use:"
    echo >&2 "cargo install --version='~0.8' sqlx-cli --no-default-features --features rustls,postgres"
    echo >&2 "to install it."
    exit 1
fi

DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

DB_NAME="${POSTGRES_DB:=pingpong}"
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=postgres}"

export PGPASSWORD="${DB_PASSWORD}"

until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

>&2 echo "Postgres is up and running on ${DB_HOST}:${DB_PORT}, running migrations now!"

DATABASE_URL=postgres://"${DB_USER}":"${DB_PASSWORD}"@"${DB_HOST}":"${DB_PORT}"/"${DB_NAME}"
export DATABASE_URL

sqlx database create
sqlx migrate run

echo "Database has been migrated successfully!"
