package main

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
)

type fakeEngineClient struct {
	response healthResponse
	err      error
}

func (c fakeEngineClient) getCheckHealth(context.Context) (healthResponse, error) {
	return c.response, c.err
}

func TestHealthHandler(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()

	newServer(nil).healthHandler(rec, req)

	assert.Equal(t, rec.Code, http.StatusOK)
	assert.Equal(t, rec.Body.String(), "{\"status\":\"ok\"}\n")
}

func TestReadyHandler(t *testing.T) {
	t.Run("engine is available", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/ready", nil)
		rec := httptest.NewRecorder()

		newServer(fakeEngineClient{response: healthResponse{Status: "engine ok"}}).readyHandler(rec, req)

		assert.Equal(t, http.StatusOK, rec.Code)
		assert.Equal(t, "{\"status\":\"engine ok\"}\n", rec.Body.String())
	})

	t.Run("engine is unavailable", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/ready", nil)
		rec := httptest.NewRecorder()

		newServer(fakeEngineClient{err: errors.New("unavailable")}).readyHandler(rec, req)

		assert.Equal(t, http.StatusInternalServerError, rec.Code)
		assert.Equal(t, "engine unavailable\n", rec.Body.String())
	})
}
