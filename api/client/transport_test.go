package client

import (
	"errors"
	"net/http"
	"reflect"
	"sync"
	"testing"
)

// roundTripFunc lets a plain function satisfy http.RoundTripper.
type roundTripFunc func(*http.Request) (*http.Response, error)

func (f roundTripFunc) RoundTrip(req *http.Request) (*http.Response, error) { return f(req) }

func newReq(t *testing.T) *http.Request {
	t.Helper()
	req, err := http.NewRequest(http.MethodGet, "http://example.invalid/queues", nil)
	if err != nil {
		t.Fatalf("NewRequest: %v", err)
	}
	return req
}

func TestWorkerTokenTransport_SetsCurrentToken(t *testing.T) {
	var got string
	next := roundTripFunc(func(req *http.Request) (*http.Response, error) {
		got = req.Header.Get(workerTokenHeader)
		return &http.Response{StatusCode: 200, Header: http.Header{}, Body: http.NoBody}, nil
	})

	tr := NewWorkerTokenTransport(next, "token-v1")
	if _, err := tr.RoundTrip(newReq(t)); err != nil {
		t.Fatalf("RoundTrip: %v", err)
	}
	if got != "token-v1" {
		t.Errorf("%s = %q, want %q", workerTokenHeader, got, "token-v1")
	}
}

func TestWorkerTokenTransport_PicksUpRefreshedToken(t *testing.T) {
	call := 0
	var used []string
	next := roundTripFunc(func(req *http.Request) (*http.Response, error) {
		call++
		used = append(used, req.Header.Get(workerTokenHeader))
		resp := &http.Response{StatusCode: 200, Header: http.Header{}, Body: http.NoBody}
		if call == 1 {
			resp.Header.Set(workerTokenRefreshHeader, "token-v2")
		}
		return resp, nil
	})

	tr := NewWorkerTokenTransport(next, "token-v1")
	if _, err := tr.RoundTrip(newReq(t)); err != nil {
		t.Fatalf("RoundTrip 1: %v", err)
	}
	if _, err := tr.RoundTrip(newReq(t)); err != nil {
		t.Fatalf("RoundTrip 2: %v", err)
	}

	if want := []string{"token-v1", "token-v2"}; !reflect.DeepEqual(used, want) {
		t.Errorf("tokens used = %v, want %v", used, want)
	}
}

func TestWorkerTokenTransport_IgnoresEmptyRefreshHeader(t *testing.T) {
	call := 0
	var used []string
	next := roundTripFunc(func(req *http.Request) (*http.Response, error) {
		call++
		used = append(used, req.Header.Get(workerTokenHeader))
		// No X-Refresh-Worker-Token set: token must carry over unchanged.
		return &http.Response{StatusCode: 200, Header: http.Header{}, Body: http.NoBody}, nil
	})

	tr := NewWorkerTokenTransport(next, "token-v1")
	if _, err := tr.RoundTrip(newReq(t)); err != nil {
		t.Fatalf("RoundTrip 1: %v", err)
	}
	if _, err := tr.RoundTrip(newReq(t)); err != nil {
		t.Fatalf("RoundTrip 2: %v", err)
	}

	if want := []string{"token-v1", "token-v1"}; !reflect.DeepEqual(used, want) {
		t.Errorf("tokens used = %v, want %v", used, want)
	}
}

func TestWorkerTokenTransport_DoesNotMutateCallerRequest(t *testing.T) {
	next := roundTripFunc(func(req *http.Request) (*http.Response, error) {
		return &http.Response{StatusCode: 200, Header: http.Header{}, Body: http.NoBody}, nil
	})
	tr := NewWorkerTokenTransport(next, "token-v1")

	req := newReq(t)
	if _, err := tr.RoundTrip(req); err != nil {
		t.Fatalf("RoundTrip: %v", err)
	}
	if h := req.Header.Get(workerTokenHeader); h != "" {
		t.Errorf("caller's request was mutated: %s = %q, want empty", workerTokenHeader, h)
	}
}

func TestWorkerTokenTransport_ErrorLeavesTokenUnchanged(t *testing.T) {
	wantErr := errors.New("boom")
	failing := true
	var used []string
	next := roundTripFunc(func(req *http.Request) (*http.Response, error) {
		used = append(used, req.Header.Get(workerTokenHeader))
		if failing {
			return nil, wantErr
		}
		return &http.Response{StatusCode: 200, Header: http.Header{}, Body: http.NoBody}, nil
	})

	tr := NewWorkerTokenTransport(next, "token-v1")
	if _, err := tr.RoundTrip(newReq(t)); !errors.Is(err, wantErr) {
		t.Fatalf("RoundTrip error = %v, want %v", err, wantErr)
	}

	failing = false
	if _, err := tr.RoundTrip(newReq(t)); err != nil {
		t.Fatalf("RoundTrip after failure: %v", err)
	}

	if want := []string{"token-v1", "token-v1"}; !reflect.DeepEqual(used, want) {
		t.Errorf("tokens used = %v, want %v (token should be unchanged after an error)", used, want)
	}
}

func TestWorkerTokenTransport_ConcurrentAccess(t *testing.T) {
	next := roundTripFunc(func(req *http.Request) (*http.Response, error) {
		resp := &http.Response{StatusCode: 200, Header: http.Header{}, Body: http.NoBody}
		resp.Header.Set(workerTokenRefreshHeader, "refreshed-"+req.Header.Get(workerTokenHeader))
		return resp, nil
	})
	tr := NewWorkerTokenTransport(next, "token-0")

	var wg sync.WaitGroup
	for i := 0; i < 50; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if _, err := tr.RoundTrip(newReq(t)); err != nil {
				t.Errorf("RoundTrip: %v", err)
			}
		}()
	}
	wg.Wait()
	// The meaningful signal here is `-race` staying clean under concurrent
	// Load/Store on t.token, not any particular final token value.
}
