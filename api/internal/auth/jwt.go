package auth

import (
	"fmt"
	"time"
	"uuid"

	"github.com/golang-jwt/jwt/v5"
)

type Claims struct {
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

func (a *Authenticator) SignWithScopes(
	subject string,
	scopes []string,
) (string, error) {
	now := time.Now()

	claims := Claims{
		Subject:   subject,
		Scopes:    scopes,
		ExpiresAt: jwt.NewNumericDate(now.Add(a.ttl)),
		IssuedAt:  jwt.NewNumericDate(now),
		NotBefore: jwt.NewNumericDate(now),
		ID:        uuid.New().String(),
		Issuer:    a.issuer,
		Audience:  []string{a.audience},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)

	signed, err := token.SignedString(a.secret)
	if err != nil {
		return "", fmt.Errorf("failed to sign string: %w", err)
	}

	return signed, nil
}
