# syntax=docker/dockerfile:1

FROM golang:1.27 AS builder

WORKDIR /src

COPY go.mod go.sum* ./
RUN go mod download

COPY . .

RUN CGO_ENABLED=0 GOOS=linux go build \
	-o /qr-api \
	./cmd/api

FROM gcr.io/distroless/static-debian12

COPY --from=builder /qr-api /qr-api

EXPOSE 8080

ENTRYPOINT ["/qr-api"]
