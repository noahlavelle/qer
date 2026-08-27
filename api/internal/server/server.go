package server

import (
	"context"
	"fmt"

	qerv1 "github.com/noahlavelle/qer/gen/qer/v1"
	"github.com/noahlavelle/qer/internal/auth"
	"google.golang.org/grpc"
)

type EngineClient struct {
	client qerv1.QueueEngineClient
}

func NewEngineClient(conn *grpc.ClientConn) *EngineClient {
	return &EngineClient{
		client: qerv1.NewQueueEngineClient(conn),
	}
}

func (e *EngineClient) CheckHealth(
	ctx context.Context,
) (*qerv1.CheckHealthResponse, error) {
	response, err := e.client.CheckHealth(ctx, &qerv1.CheckHealthRequest{})
	if err != nil {
		return response, fmt.Errorf("get engine health: %w", err)
	}

	return response, nil
}

type Server struct {
	engine *EngineClient
	auth   *auth.Authenticator
}

func NewServer(engine *EngineClient, auth *auth.Authenticator) *Server {
	return &Server{
		engine,
		auth,
	}
}
