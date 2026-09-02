package server

import (
	"context"
	"fmt"
	"time"

	qerv1 "github.com/noahlavelle/qer/gen/qer/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/health/grpc_health_v1"
)

type EngineClient struct {
	client       qerv1.QueueEngineClient
	healthClient grpc_health_v1.HealthClient
}

func NewEngineClient(conn *grpc.ClientConn) *EngineClient {
	return &EngineClient{
		client:       qerv1.NewQueueEngineClient(conn),
		healthClient: grpc_health_v1.NewHealthClient(conn),
	}
}

func (e *EngineClient) CheckHealth(
	ctx context.Context,
) (*grpc_health_v1.HealthCheckResponse, error) {
	ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
	defer cancel()

	res, err := e.healthClient.Check(ctx, &grpc_health_v1.HealthCheckRequest{})
	if err != nil {
		return res, fmt.Errorf("get engine health: %w", err)
	}
	if res.GetStatus() != grpc_health_v1.HealthCheckResponse_SERVING {
		return res, fmt.Errorf("engine reported status %s", res.GetStatus())
	}

	return res, nil
}

func (e *EngineClient) CreateQueue(
	ctx context.Context,
	name string,
) (*qerv1.CreateQueueResponse, error){
	res, err := e.client.CreateQueue(ctx, &qerv1.CreateQueueRequest{
		Name: name,
	})
	if err != nil {
		return res, fmt.Errorf("create queue: %w", err)
	}

	return res, nil
}

func (e *EngineClient) Put(
	ctx context.Context,
	queueName string,
	payload []byte,
) (*qerv1.PutResponse, error) {
	res, err := e.client.Put(ctx, &qerv1.PutRequest{
		QueueName: queueName,
		Payload: payload,
	})
	if err != nil {
		return res, fmt.Errorf("put job: %w", err)
	}

	return res, nil
}

func (e *EngineClient) Reserve(
	ctx context.Context,
	queueName string,
) (*qerv1.ReserveResponse, error) {
	res, err := e.client.Reserve(ctx, &qerv1.ReserveRequest{
		QueueName: queueName,
	})
	if err != nil {
		return res, fmt.Errorf("reserve job: %w", err)
	}

	return res, nil
}

func (e *EngineClient) Ack(
	ctx context.Context,
	reservationId string,
) (*qerv1.AckResponse, error) {
	res, err := e.client.Ack(ctx, &qerv1.AckRequest{
		ReservationId: reservationId,
	})
	if err != nil {
		return res, fmt.Errorf("ack job: %w", err)
	}

	return res, nil
}
