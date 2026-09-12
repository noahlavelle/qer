package main

import (
	"log"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/noahlavelle/qer/gen/openapi"
	"github.com/noahlavelle/qer/internal/auth"
	"github.com/noahlavelle/qer/internal/server"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func main() {
	engineAddress := os.Getenv("ENGINE_ADDR")
	if engineAddress == "" {
		engineAddress = "engine:50051"
	}

	conn, err := grpc.NewClient(
		engineAddress,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithUnaryInterceptor(server.WorkerTokenInterceptor),
	)
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	engineClient := server.NewEngineClient(conn)
	authenticator := auth.NewAuthenticator(
		[]byte("secret"),
		"qer-api",
		"qer-engine",
		5*time.Minute,
	)

	srv := server.NewServer(engineClient, authenticator)
	handler := openapi.NewStrictHandler(srv, []openapi.StrictMiddlewareFunc{
		srv.WorkerTokenMiddleware,
	})

	r := chi.NewRouter()
	h := openapi.HandlerWithOptions(handler, openapi.ChiServerOptions{
		BaseRouter: r,
		BaseURL:    "/api/v1",
	})
	s := &http.Server{
		Handler: h,
		Addr:    "0.0.0.0:8080",
	}

	log.Println("API listening on :8080")

	log.Fatal(s.ListenAndServe())
}
