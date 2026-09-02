package server

import (
	"context"
	"fmt"
	"net/http"

	"github.com/noahlavelle/qer/gen/openapi"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

type workerTokenKey struct{}

func WorkerTokenMiddleware(
	next openapi.StrictHandlerFunc,
	operationID string,
) openapi.StrictHandlerFunc {
	return func(
		ctx context.Context,
		w http.ResponseWriter,
		r *http.Request,
		request any,
	) (any, error) {
		switch operationID {
		case "CreateQueue", "PutJob", "ReserveJob", "AckJob":
			token := r.Header.Get("X-Worker-Token")
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
	if method != grpc_health_v1.Health_Check_FullMethodName {
		token, ok := ctx.Value(workerTokenKey{}).(string)
		if !ok {
			return status.Error(
				codes.Unauthenticated,
				"missing worker token",
			)
		}

		ctx = metadata.AppendToOutgoingContext(
			ctx,
			"x-worker-token",
			token,
		)
	}

	return invoker(ctx, method, req, reply, conn, opts...)
}
