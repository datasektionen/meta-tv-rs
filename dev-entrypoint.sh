#!/bin/sh
# Entrypoint for the development container (see Dockerfile.dev and compose.yml).
#
# Rebuilds the frontend and backend whenever their source changes:
#  - nginx listens on port 8000 and forwards requests to the backend, except
#    for trunk's auto-reload websocket which goes to trunk.
#  - trunk rebuilds the frontend into /www/static (which the backend serves)
#    and tells the browser to reload once it's done.
#  - watchexec rebuilds and restarts the backend.
#
# The frontend and backend use separate target directories so that they can be
# built concurrently.

set -e

nginx

(
    cd crates/frontend
    CARGO_TARGET_DIR=/app/target/frontend exec trunk serve \
        --config Trunk.dev.toml \
        --dist /www/static \
        --address 127.0.0.1 \
        --port 8080 \
        --watch . \
        --watch ../common \
        --skip-version-check
) &

CARGO_TARGET_DIR=/app/target/backend exec watchexec \
    --restart \
    --watch crates/backend \
    --watch crates/common \
    --watch crates/entity \
    --watch crates/migration \
    --watch Cargo.toml \
    --watch Cargo.lock \
    -- cargo run --package meta-tv-rs
