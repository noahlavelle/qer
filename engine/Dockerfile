# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.98
ARG APP_NAME=qer-engine

FROM scratch AS skaffold-dependencies
COPY engine/src /engine/src
COPY engine/build.rs engine/Cargo.toml engine/Cargo.lock /engine/
COPY proto /proto

FROM docker.io/library/rust:${RUST_VERSION}-alpine AS build

ARG APP_NAME

WORKDIR /app

RUN apk update && apk add --no-cache clang lld musl-dev git protoc

RUN --mount=type=bind,source=engine/src,target=src \
    --mount=type=bind,source=engine/build.rs,target=build.rs \
    --mount=type=bind,source=engine/Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=engine/Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=engine/migrations,target=migrations \
    --mount=type=bind,source=proto,target=proto \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/var/cache/cargo \
    CARGO_HOME=/var/cache/cargo cargo build --locked --release && \
    cp ./target/release/${APP_NAME} /bin/engine

FROM docker.io/library/alpine:3.23.5 AS release

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

USER appuser

COPY --from=build bin/engine /bin

EXPOSE 50051

ENTRYPOINT ["/bin/engine"]
