package server

import (
	"context"
	"uuid"

	"github.com/noahlavelle/qer/gen/openapi"
)

var _ openapi.StrictServerInterface = (*Server)(nil)

func (s *Server) CheckHealth(
	ctx context.Context,
	request openapi.CheckHealthRequestObject,
) (openapi.CheckHealthResponseObject, error) {
	return openapi.CheckHealth200JSONResponse{
		Status: "ok",
	}, nil
}

func (s *Server) CheckReady(
	ctx context.Context,
	request openapi.CheckReadyRequestObject,
) (openapi.CheckReadyResponseObject, error) {
	res, err := s.engine.CheckHealth(ctx)
	if err != nil {
		return openapi.CheckReady503JSONResponse{}, err
	}

	return openapi.CheckReady200JSONResponse{
		Status: openapi.ReadyResponseStatus(res.Status),
	}, nil
}

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
		return nil, err
	}

	return openapi.AttachConsumer200JSONResponse{
		Token: token,
	}, nil
}

func (s *Server) AttachProducer(
	ctx context.Context,
	request openapi.AttachProducerRequestObject,
) (openapi.AttachProducerResponseObject, error) {
	token, err := s.auth.SignWithScopes(
		uuid.NewV4().String(),
		[]string{"queue.put"},
	)
	if err != nil {
		return nil, err
	}

	return openapi.AttachProducer200JSONResponse{
		Token: token,
	}, nil
}

// CreateQueue implements [openapi.StrictServerInterface].
func (s *Server) CreateQueue(ctx context.Context, request openapi.CreateQueueRequestObject) (openapi.CreateQueueResponseObject, error) {
	panic("unimplemented")
}

// PutJob implements [openapi.StrictServerInterface].
func (s *Server) PutJob(ctx context.Context, request openapi.PutJobRequestObject) (openapi.PutJobResponseObject, error) {
	panic("unimplemented")
}

// ReserveJob implements [openapi.StrictServerInterface].
func (s *Server) ReserveJob(ctx context.Context, request openapi.ReserveJobRequestObject) (openapi.ReserveJobResponseObject, error) {
	panic("unimplemented")
}
