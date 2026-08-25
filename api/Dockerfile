# syntax=docker/dockerfile:1

ARG GO_VERSION=1.27

FROM scratch AS skaffold-dependencies
COPY api/cmd /api/cmd
COPY api/gen /api/gen
COPY api/go.mod api/go.sum /api/

FROM docker.io/library/golang:${GO_VERSION}-alpine AS build

ARG APP_NAME

WORKDIR /app

RUN apk update && apk add --no-cache git

RUN --mount=type=bind,source=api/cmd,target=cmd \
    --mount=type=bind,source=api/internal,target=internal \
    --mount=type=bind,source=api/go.mod,target=go.mod \
    --mount=type=bind,source=api/go.sum,target=go.sum \
    --mount=type=bind,source=api/gen,target=gen \
    --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go build -o /bin/api ./cmd/api

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

COPY --from=build /bin/api /bin/api

EXPOSE 8080

ENTRYPOINT ["/bin/api"]
