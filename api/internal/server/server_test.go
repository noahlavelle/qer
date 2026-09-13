package server

import (
	"context"
	"errors"
	"testing"

	"github.com/noahlavelle/qer/gen/openapi"
	qerv1 "github.com/noahlavelle/qer/gen/qer/v1"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

type fakeQueueEngineClient struct {
	qerv1.QueueEngineClient
	createQueueErr error
}

func (f *fakeQueueEngineClient) CreateQueue(
	ctx context.Context,
	in *qerv1.CreateQueueRequest,
	opts ...grpc.CallOption,
) (*qerv1.CreateQueueResponse, error) {
	if f.createQueueErr != nil {
		return nil, f.createQueueErr
	}
	return &qerv1.CreateQueueResponse{}, nil
}

func newTestServer(client qerv1.QueueEngineClient) *Server {
	return &Server{engine: &EngineClient{client: client}}
}

func TestCreateQueue_AlreadyExistsMapsTo409(t *testing.T) {
	s := newTestServer(&fakeQueueEngineClient{
		createQueueErr: status.Error(codes.AlreadyExists, "queue already exists: emails"),
	})

	res, err := s.CreateQueue(context.Background(), openapi.CreateQueueRequestObject{
		Body: &openapi.CreateQueueJSONRequestBody{Name: "emails"},
	})

	require.NoError(t, err)
	conflict, ok := res.(openapi.CreateQueue409JSONResponse)
	require.True(t, ok, "expected CreateQueue409JSONResponse, got %T", res)
	assert.Equal(t, "queue_exists", conflict.Body.Code)
}

func TestCreateQueue_OtherErrorMapsTo500(t *testing.T) {
	s := newTestServer(&fakeQueueEngineClient{
		createQueueErr: errors.New("boom"),
	})

	res, err := s.CreateQueue(context.Background(), openapi.CreateQueueRequestObject{
		Body: &openapi.CreateQueueJSONRequestBody{Name: "emails"},
	})

	assert.Error(t, err)
	_, ok := res.(openapi.CreateQueue500JSONResponse)
	assert.True(t, ok, "expected CreateQueue500JSONResponse, got %T", res)
}

func TestCreateQueue_Success(t *testing.T) {
	s := newTestServer(&fakeQueueEngineClient{})

	res, err := s.CreateQueue(context.Background(), openapi.CreateQueueRequestObject{
		Body: &openapi.CreateQueueJSONRequestBody{Name: "emails"},
	})

	require.NoError(t, err)
	created, ok := res.(openapi.CreateQueue201JSONResponse)
	require.True(t, ok, "expected CreateQueue201JSONResponse, got %T", res)
	assert.Equal(t, "emails", created.Body.Name)
}
