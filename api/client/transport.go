package client

import (
	"net/http"
	"sync/atomic"
)

const (
	workerTokenHeader        = "X-Worker-Token"
	workerTokenRefreshHeader = "X-Refresh-Worker-Token"
)

// WorkerTokenTransport attaches the current worker JWT to every outgoing
// request, and automatically swaps in the refreshed token after expiry.
type WorkerTokenTransport struct {
	next http.RoundTripper
	token atomic.Value
}

// NewWorkerTokenTransport returns a new worker token transport.
func NewWorkerTokenTransport(
	next http.RoundTripper,
	initialToken string,
) *WorkerTokenTransport {
	if next == nil {
		next = http.DefaultTransport
	}

	t := &WorkerTokenTransport{next: next}
	t.token.Store(initialToken)
	return t
}

// RoundTrip attaches and refreshes worker JWT's in outgoing requests, leaving
// the input request unmodified.
func (t *WorkerTokenTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	// Clone as to not mutate input request
	req = req.Clone(req.Context())
	req.Header.Set(workerTokenHeader, t.token.Load().(string))

	resp, err := t.next.RoundTrip(req)
	if err != nil {
		return resp, err
	}

	if refreshed := resp.Header.Get(workerTokenRefreshHeader); refreshed != "" {
		t.token.Store(refreshed)
	}

	return resp, nil
}
