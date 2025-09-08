########################### frontend ###################################

FROM rust:1.89-alpine AS frontend-build
WORKDIR /build

RUN apk update && apk upgrade && apk add make alpine-sdk libffi-dev trunk gcompat build-base

RUN wget https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64-musl
RUN chmod +x tailwindcss-linux-x64-musl
RUN mv tailwindcss-linux-x64-musl /usr/bin/tailwindcss
RUN tailwindcss

RUN rustup target add wasm32-unknown-unknown

COPY crates/entity/ ../entity/
COPY crates/common/ ../common/

COPY crates/frontend/Cargo.toml .
COPY crates/frontend/Trunk.toml Trunk.toml
RUN mkdir src
COPY crates/frontend/dev/main.rs src/main.rs
COPY crates/frontend/dev/index.html index.html

RUN trunk build

COPY crates/frontend/public/ public/
COPY crates/frontend/src/ src/
COPY crates/frontend/index.html index.html

RUN trunk build


######################## backend #########################################

FROM rust:1.89-alpine AS build
WORKDIR /build

RUN apk update && apk upgrade && apk add make alpine-sdk libffi-dev

COPY crates/migration/ ../migration/
COPY crates/entity/ ../entity/
COPY crates/common/ ../common/
COPY crates/backend/Cargo.toml .
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

RUN cargo build -r

COPY crates/backend/ .

RUN cargo build -r

FROM alpine:latest

WORKDIR /srv

COPY --from=frontend-build /build/dist/ /www/static/
COPY --from=build /build/target/release/meta-tv-rs meta-tv-rs

RUN mkdir /srv/uploads

CMD ["./meta-tv-rs"]
