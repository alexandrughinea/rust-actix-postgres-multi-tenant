#!/usr/bin/env bash

set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo "psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo "sqlx is not installed."
  exit 1
fi

# Check if a custom user has been set, otherwise default to "postgres".
DB_USER="${POSTGRES_USER:=postgres}"

# Check if a custom password has been set, otherwise default to "password".
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"

# Check if a custom database name has been set, otherwise default to "newsletter".
DB_NAME="${POSTGRES_DB:=newsletter}"

# Check if a custom database port has been set, otherwise default to "5432".
DB_PORT="${POSTGRES_PORT:=5432}"

# Check if a custom database host has been set, otherwise default to "localhost".
DB_HOST="${POSTGRES_HOST:=localhost}"

# Allow to skip Docker if a dockerized postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  #Launch postgres with Docker
  docker run \
  -e POSTGRES_USER=${DB_USER} \
  -e POSTGRES_PASSWORD=${DB_PASSWORD} \
  -e POSTGRES_DB=${DB_NAME} \
  -p "${DB_PORT}":5432 \
  -d postgres \
  postgres -N 1000
fi

#Keep pinging postgres until it is ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    echo "Postgres is still unavailable, sleeping ..."
    sleep 1
done

echo "Postgres is up and running on port #{DB_PORT}"

DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
export DATABASE_URL
sqlx database create

echo "Running migrations now"
sqlx migrate run

echo "Postgres has been migrated, all good!"
