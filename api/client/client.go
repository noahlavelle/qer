package client

import (
	"context"
	"fmt"
	"net/http"

	"github.com/noahlavelle/qer/gen/openapi"
)

// New creates a new client with the given url and token. It will not be able
// to call engine operations unless the caller already has a signed worker JWT.
func New(baseURL string, initialToken string) (*openapi.ClientWithResponses, error) {
	t := NewWorkerTokenTransport(http.DefaultTransport, initialToken)

	client, err := openapi.NewClientWithResponses(
		baseURL,
		openapi.WithHTTPClient(&http.Client{Transport: t}),
	)
	if err != nil {
		return nil, fmt.Errorf("create client: %w", err)
	}

	return client, nil
}

// AttachConsumer creates a new client registered as a consumer.
func AttachConsumer(ctx context.Context, baseURL string) (*openapi.ClientWithResponses, error) {
	bootstrap, err := openapi.NewClientWithResponses(baseURL)
	if err != nil {
		return nil, fmt.Errorf("bootstrap client: %w", err)
	}
	res, err := bootstrap.AttachConsumerWithResponse(ctx)
	if err != nil {
		return nil, fmt.Errorf("attach consumer: %w", err)
	}
	if res.JSON200 == nil {
		return nil, fmt.Errorf(
			"attach consumer: unexpected status %s",
			res.Status(),
		)
	}
	return New(baseURL, res.JSON200.Token)
}

// AttachProducer creates a new client registered as a producer.
func AttachProducer(ctx context.Context, baseURL string) (*openapi.ClientWithResponses, error) {
	bootstrap, err := openapi.NewClientWithResponses(baseURL)
	if err != nil {
		return nil, fmt.Errorf("bootstrap client: %w", err)
	}
	res, err := bootstrap.AttachProducerWithResponse(ctx)
	if err != nil {
		return nil, fmt.Errorf("attach consumer: %w", err)
	}
	if res.JSON200 == nil {
		return nil, fmt.Errorf(
			"attach consumer: unexpected status %s",
			res.Status(),
		)
	}
	return New(baseURL, res.JSON200.Token)
}
