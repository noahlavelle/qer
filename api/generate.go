package qer

//go:generate protoc --proto_path=../proto --go_out=./gen --go_opt=paths=source_relative --go-grpc_out=./gen --go-grpc_opt=paths=source_relative ../proto/qer/v1/engine.proto
//go:generate go tool vacuum lint openapi.yaml
//go:generate go tool oapi-codegen --config=config.yaml -o gen/openapi/qer-api.go ./openapi.yaml
