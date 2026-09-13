package server

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	qerv1 "github.com/noahlavelle/qer/gen/qer/v1"
	"github.com/noahlavelle/qer/internal/auth"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const refreshWorkerTokenHeader = "X-Refresh-Worker-Token"

func newTestAuthenticator(ttl time.Duration) *auth.Authenticator {
	return auth.NewAuthenticator([]byte("test-secret"), "qer-api", "qer-engine", ttl)
}

func TestWorkerTokenMiddleware_NonWorkerAuthedOperation(t *testing.T) {
	s := &Server{auth: newTestAuthenticator(5 * time.Minute)}

	called := false
	next := func(ctx context.Context, w http.ResponseWriter, r *http.Request, request any) (any, error) {
		called = true
		return nil, nil
	}

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()

	_, err := s.WorkerTokenMiddleware(next, "CheckHealth")(context.Background(), rec, req, nil)

	require.NoError(t, err)
	assert.True(t, called)
	assert.Empty(t, rec.Header().Get(refreshWorkerTokenHeader))
}

func TestWorkerTokenMiddleware_MissingToken(t *testing.T) {
	s := &Server{auth: newTestAuthenticator(5 * time.Minute)}

	called := false
	next := func(ctx context.Context, w http.ResponseWriter, r *http.Request, request any) (any, error) {
		called = true
		return nil, nil
	}

	req := httptest.NewRequest(http.MethodPost, "/queues", nil)
	rec := httptest.NewRecorder()

	_, err := s.WorkerTokenMiddleware(next, "CreateQueue")(context.Background(), rec, req, nil)

	assert.Error(t, err)
	assert.False(t, called)
	assert.Empty(t, rec.Header().Get(refreshWorkerTokenHeader))
}

func TestWorkerTokenMiddleware_InvalidToken(t *testing.T) {
	s := &Server{auth: newTestAuthenticator(5 * time.Minute)}

	called := false
	next := func(ctx context.Context, w http.ResponseWriter, r *http.Request, request any) (any, error) {
		called = true
		return nil, nil
	}

	req := httptest.NewRequest(http.MethodPost, "/queues", nil)
	req.Header.Set("X-Worker-Token", "not-a-real-token")
	rec := httptest.NewRecorder()

	_, err := s.WorkerTokenMiddleware(next, "CreateQueue")(context.Background(), rec, req, nil)

	assert.Error(t, err)
	assert.False(t, called)
	assert.Empty(t, rec.Header().Get(refreshWorkerTokenHeader))
}

func TestWorkerTokenMiddleware_ValidUnexpiredToken(t *testing.T) {
	s := &Server{auth: newTestAuthenticator(5 * time.Minute)}

	token, err := s.auth.SignWithScopes("worker-1", []qerv1.Scope{qerv1.Scope_QUEUE_PRODUCE})
	require.NoError(t, err)

	var gotToken string
	next := func(ctx context.Context, w http.ResponseWriter, r *http.Request, request any) (any, error) {
		gotToken, _ = ctx.Value(workerTokenKey{}).(string)
		return nil, nil
	}

	req := httptest.NewRequest(http.MethodPost, "/queues", nil)
	req.Header.Set("X-Worker-Token", token)
	rec := httptest.NewRecorder()

	_, err = s.WorkerTokenMiddleware(next, "CreateQueue")(context.Background(), rec, req, nil)

	require.NoError(t, err)
	assert.Equal(t, token, gotToken)
	assert.Empty(t, rec.Header().Get(refreshWorkerTokenHeader))
}

func TestWorkerTokenMiddleware_ExpiredTokenIsRefreshed(t *testing.T) {
	s := &Server{auth: newTestAuthenticator(5 * time.Minute)}

	expiredTokenIssuer := newTestAuthenticator(-1 * time.Minute)
	expiredToken, err := expiredTokenIssuer.SignWithScopes("worker-1", []qerv1.Scope{qerv1.Scope_QUEUE_PRODUCE})
	require.NoError(t, err)

	var gotToken string
	next := func(ctx context.Context, w http.ResponseWriter, r *http.Request, request any) (any, error) {
		gotToken, _ = ctx.Value(workerTokenKey{}).(string)
		return nil, nil
	}

	req := httptest.NewRequest(http.MethodPost, "/queues", nil)
	req.Header.Set("X-Worker-Token", expiredToken)
	rec := httptest.NewRecorder()

	_, err = s.WorkerTokenMiddleware(next, "CreateQueue")(context.Background(), rec, req, nil)

	require.NoError(t, err)

	refreshedToken := rec.Header().Get(refreshWorkerTokenHeader)
	assert.NotEmpty(t, refreshedToken)
	assert.NotEqual(t, expiredToken, refreshedToken)
	assert.Equal(t, refreshedToken, gotToken)

	claims, err := s.auth.Parse(refreshedToken)
	require.NoError(t, err)
	assert.True(t, claims.ExpiresAt.Time.After(time.Now()))
	assert.Equal(t, "worker-1", claims.Subject)
	assert.Equal(t, []qerv1.Scope{qerv1.Scope_QUEUE_PRODUCE}, claims.Scopes)
}

func TestWorkerTokenMiddleware_AppliesToAllWorkerAuthedOperations(t *testing.T) {
	s := &Server{auth: newTestAuthenticator(5 * time.Minute)}

	expiredTokenIssuer := newTestAuthenticator(-1 * time.Minute)
	token, err := expiredTokenIssuer.SignWithScopes("worker-1", []qerv1.Scope{qerv1.Scope_QUEUE_PRODUCE})
	require.NoError(t, err)

	for _, operationID := range []string{"CreateQueue", "PutJob", "ReserveJob", "AckJob"} {
		t.Run(operationID, func(t *testing.T) {
			next := func(ctx context.Context, w http.ResponseWriter, r *http.Request, request any) (any, error) {
				return nil, nil
			}

			req := httptest.NewRequest(http.MethodPost, "/queues", nil)
			req.Header.Set("X-Worker-Token", token)
			rec := httptest.NewRecorder()

			_, err := s.WorkerTokenMiddleware(next, operationID)(context.Background(), rec, req, nil)

			require.NoError(t, err)
			assert.NotEmpty(t, rec.Header().Get(refreshWorkerTokenHeader))
		})
	}
}
