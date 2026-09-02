package server

import (
	"context"
	"encoding/json"
	"fmt"
	"uuid"

	"github.com/noahlavelle/qer/gen/openapi"
	"github.com/noahlavelle/qer/internal/auth"
	openapi_types "github.com/oapi-codegen/runtime/types"
)

type Server struct {
	engine *EngineClient
	auth   *auth.Authenticator
}

var _ openapi.StrictServerInterface = (*Server)(nil)

func NewServer(engine *EngineClient, auth *auth.Authenticator) *Server {
	return &Server{
		engine,
		auth,
	}
}

// CheckHealth implements [openapi.StrictServerInterface].
func (s *Server) CheckHealth(
	ctx context.Context,
	request openapi.CheckHealthRequestObject,
) (openapi.CheckHealthResponseObject, error) {
	return openapi.CheckHealth200JSONResponse{
		Status: "ok",
	}, nil
}

// CheckReady implements [openapi.StrictServerInterface].
func (s *Server) CheckReady(
	ctx context.Context,
	request openapi.CheckReadyRequestObject,
) (openapi.CheckReadyResponseObject, error) {
	if _, err := s.engine.CheckHealth(ctx); err != nil {
		return openapi.CheckReady503JSONResponse{},
			fmt.Errorf("check engine readiness: %w", err)
	}

	return openapi.CheckReady200JSONResponse{
		Status: openapi.ReadyResponseStatusOk,
	}, nil
}

// AttachConsumer implements [openapi.StrictServerInterface].
//
// TODO: Make these functions idempotent (reuse a token already in headers?)
func (s *Server) AttachConsumer(
	ctx context.Context,
	request openapi.AttachConsumerRequestObject,
) (openapi.AttachConsumerResponseObject, error) {
	token, err := s.auth.SignWithScopes(
		uuid.NewV4().String(),
		[]string{"queue.reserve", "queue.ack"},
	)
	if err != nil {
		return openapi.AttachConsumer500JSONResponse{},
			fmt.Errorf("create worker token: %w", err)
	}

	return openapi.AttachConsumer200JSONResponse{
		Token: token,
	}, nil
}

// AttachProducer implements [openapi.StrictServerInterface].
func (s *Server) AttachProducer(
	ctx context.Context,
	request openapi.AttachProducerRequestObject,
) (openapi.AttachProducerResponseObject, error) {
	token, err := s.auth.SignWithScopes(
		uuid.NewV4().String(),
		[]string{"queue.put"},
	)
	if err != nil {
		return openapi.AttachProducer500JSONResponse{},
			fmt.Errorf("create worker token: %w", err)
	}

	return openapi.AttachProducer200JSONResponse{
		Token: token,
	}, nil
}

// CreateQueue implements [openapi.StrictServerInterface].
func (s *Server) CreateQueue(
	ctx context.Context,
	request openapi.CreateQueueRequestObject,
) (openapi.CreateQueueResponseObject, error) {
	if _, err := s.engine.CreateQueue(ctx, request.Body.Name); err != nil {
		return openapi.CreateQueue500JSONResponse{},
			fmt.Errorf("create queue: %w", err)
	}

	return openapi.CreateQueue201JSONResponse{}, nil
}

// PutJob implements [openapi.StrictServerInterface].
func (s *Server) PutJob(
	ctx context.Context,
	request openapi.PutJobRequestObject,
) (openapi.PutJobResponseObject, error) {
	payload, err := json.Marshal(request.Body.Payload)
	if err != nil {
		return openapi.PutJob500JSONResponse{},
			fmt.Errorf("marshal payload: %w", err)
	}

	res, err := s.engine.Put(ctx, request.QueueName, payload)
	if err != nil {
		return openapi.PutJob500JSONResponse{},
			fmt.Errorf("register job: %w", err)
	}

	jobId, err := uuid.Parse(res.JobId)
	if err != nil {
		return openapi.PutJob500JSONResponse{},
			fmt.Errorf("parse job id: %w", err)
	}

	return openapi.PutJob202JSONResponse{
		JobId: openapi_types.UUID(jobId),
	}, nil
}

// ReserveJob implements [openapi.StrictServerInterface].
func (s *Server) ReserveJob(
	ctx context.Context,
	request openapi.ReserveJobRequestObject,
) (openapi.ReserveJobResponseObject, error) {
	res, err := s.engine.Reserve(ctx, request.QueueName)
	if err != nil {
		return openapi.ReserveJob500JSONResponse{},
			fmt.Errorf("reserve job: %w", err)
	}

	reservation := res.Reservation
	if reservation == nil {
		return openapi.ReserveJob404JSONResponse{}, nil
	}

	jobId, err := uuid.Parse(reservation.JobId)
	if err != nil {
		return openapi.ReserveJob500JSONResponse{},
			fmt.Errorf("parse job id: %w", err)
	}

	leaseId, err := uuid.Parse(reservation.ReservationId)
	if err != nil {
		return openapi.ReserveJob500JSONResponse{},
			fmt.Errorf("parse lease id: %w", err)
	}

	var payload map[string]any
	if err := json.Unmarshal(reservation.Payload, &payload); err != nil {
		return openapi.ReserveJob500JSONResponse{},
			fmt.Errorf("unmarshal payload: %w", err)
	}

	return openapi.ReserveJob200JSONResponse{
		JobId:   openapi_types.UUID(jobId),
		LeaseId: openapi_types.UUID(leaseId),
		Payload: payload,
	}, nil
}

// AckJob implements [openapi.StrictServerInterface].
func (s *Server) AckJob(
	ctx context.Context,
	request openapi.AckJobRequestObject,
) (openapi.AckJobResponseObject, error) {
	_, err := s.engine.Ack(ctx, request.Body.LeaseId.String())
	if err != nil {
		return openapi.AckJob500JSONResponse{},
			fmt.Errorf("ack job: %w", err)
	}

	return openapi.AckJob204Response{}, nil
}
