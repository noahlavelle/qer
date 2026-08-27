package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/noahlavelle/qer/gen/openapi"
	"github.com/noahlavelle/qer/internal/auth"
	"github.com/noahlavelle/qer/internal/server"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

type workerTokenKey struct {}

func WorkerTokenMiddleware(
	next openapi.StrictHandlerFunc,
	operationID string,
) openapi.StrictHandlerFunc {
	return func(
		ctx context.Context,
		w http.ResponseWriter,
		r* http.Request,
		request any,
	) (any, error) {
		switch operationID {
		case "createQueue", "putJob", "reserveJob", "ackJob":
			token := r.Header.Get("Authorization")
			if token == "" {
				return nil, fmt.Errorf("missing worker authorization")
			}

			ctx = context.WithValue(
				ctx,
				workerTokenKey{},
				token,
			)
		}

		return next(ctx, w, r, request)
	}
}

func WorkerTokenInterceptor(
	ctx context.Context,
	method string,
	req any,
	reply any,
	conn *grpc.ClientConn,
	invoker grpc.UnaryInvoker,
	opts ...grpc.CallOption,
) error {
	if method != "/qer.v1.QueueEngine/CheckHealth" {
		token, ok := ctx.Value(workerTokenKey{}).(string)
		if !ok {
			return status.Error(
				codes.Unauthenticated,
				"missing worker token",
			)
		}

		ctx = metadata.AppendToOutgoingContext(
			ctx,
			"authorization",
			token,
		)
	}

	return invoker(ctx, method, req, reply, conn, opts...)
}

func main() {
	engineAddress := os.Getenv("ENGINE_ADDR")
	if engineAddress == "" {
		engineAddress = "engine:50051"
	}

	conn, err := grpc.NewClient(
		engineAddress,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithUnaryInterceptor(WorkerTokenInterceptor),
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

	server := server.NewServer(engineClient, authenticator)
	handler := openapi.NewStrictHandler(server, []openapi.StrictMiddlewareFunc{
		WorkerTokenMiddleware,
	})

	r := chi.NewRouter()
	h := openapi.HandlerFromMux(handler, r)
	s := &http.Server{
		Handler: h,
		Addr:    "0.0.0.0:8080",
	}

	log.Println("API listening on :8080")

	log.Fatal(s.ListenAndServe())
}
