########################### frontend ###################################

FROM rust:1.89-alpine AS frontend-build
WORKDIR /build

RUN apk update && apk upgrade && apk add make alpine-sdk libffi-dev trunk gcompat build-base

RUN wget https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64-musl
RUN chmod +x tailwindcss-linux-x64-musl
RUN mv tailwindcss-linux-x64-musl /usr/bin/tailwindcss
RUN tailwindcss

RUN rustup target add wasm32-unknown-unknown

COPY crates/entity/ /build/crates/entity/
WORKDIR /build/crates/entity

RUN cargo build -r

COPY crates/common/ /build/crates/common/
WORKDIR /build/crates/common

RUN cargo build -r

COPY crates/frontend/ /build/crates/frontend/
WORKDIR /build/crates/frontend

RUN trunk build

######################## backend #########################################

FROM rust:1.89-alpine AS build
WORKDIR /build

RUN apk update && apk upgrade && apk add make alpine-sdk libffi-dev

COPY crates/migration/ /build/crates/migration/
WORKDIR /build/crates/migration
RUN cargo build -r

COPY crates/entity/ /build/crates/entity/
WORKDIR /build/crates/entity
RUN cargo build -r

COPY crates/common/ /build/crates/common/
WORKDIR /build/crates/entity
RUN cargo build -r

COPY crates/backend/ /build/crates/backend/
WORKDIR /build/crates/backend

RUN cargo build -r

FROM alpine:latest

WORKDIR /srv

COPY --from=frontend-build /build/crates/frontend/dist/ /www/static/
COPY --from=build /build/crates/backend/target/release/meta-tv-rs meta-tv-rs

RUN mkdir upload

CMD ["./meta-tv-rs"]
