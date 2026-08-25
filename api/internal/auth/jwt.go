package auth

import (
	"context"
	"fmt"
	"time"
	"uuid"

	"github.com/golang-jwt/jwt/v5"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

type Claims struct {
	WorkerID string   `json:"worker_id"`
	Scopes   []string `json:"scopes"`
	jwt.RegisteredClaims
}

type Authenticator struct {
	secret   []byte
	ttl      time.Duration
	audience string
	issuer   string
}

func NewAuthenticator(
	secret []byte,
	issuer string,
	audience string,
	ttl time.Duration,
) *Authenticator{
	return &Authenticator{
		secret,
		ttl,
		audience,
		issuer,
	}
}

func (a Authenticator) SignForRequest(
	ctx context.Context,
	method string,
	request any,
) (string, error) {
	workerID := "0001"

	return a.sign(
		workerID,
		[]string{"testscope"},
	)
}

func (a *Authenticator) sign(
	workerID string,
	scopes []string,
) (string, error) {
	now := time.Now()

	claims := Claims{
		WorkerID:  workerID,
		Scopes: scopes,
		ExpiresAt:  jwt.NewNumericDate(now.Add(a.ttl)),
		IssuedAt:   jwt.NewNumericDate(now),
		NotBefore:  jwt.NewNumericDate(now),
		ID:         uuid.New().String(),
		Issuer:     a.issuer,
		Subject:    workerID,
		Audience: 	[]string{a.audience},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)

	signed, err := token.SignedString(a.secret)
	if err != nil {
		return "", fmt.Errorf("failed to sign string: %w", err)
	}

	return signed, nil
}

type TokenSource func(
	ctx context.Context,
	method string,
	request any,
) (string, error)

func UnaryAuthInterceptor(
	tokenSource TokenSource,
) grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req any,
		reply any,
		conn *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
			token, err := tokenSource(ctx, method, req)
			if err != nil {
				return fmt.Errorf("create auth token: %w", err)
			}

			ctx = metadata.AppendToOutgoingContext(
				ctx,
				"authorization",
				"Bearer "+token,
			)

			return invoker(ctx, method, req, reply, conn, opts...)

	}
}
