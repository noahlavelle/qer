package auth

import (
	"errors"
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
		ExpiresAt: a.GetExpiry(now),
		IssuedAt:  jwt.NewNumericDate(now),
		NotBefore: jwt.NewNumericDate(now),
		ID:        uuid.New().String(),
		Issuer:    a.issuer,
		Audience:  []string{a.audience},
	}

	return a.SignClaims(claims)
}

func (a *Authenticator) SignClaims(claims Claims) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)

	signed, err := token.SignedString(a.secret)
	if err != nil {
		return "", fmt.Errorf("failed to sign string: %w", err)
	}

	return signed, nil
}

func (a *Authenticator) GetExpiry(now time.Time) *jwt.NumericDate {
	return jwt.NewNumericDate(now.Add(a.ttl))
}

func (a *Authenticator) Parse(token string) (Claims, error) {
	claims := Claims{}

	_, err := jwt.ParseWithClaims(token, &claims, func(_ *jwt.Token) (any, error) {
		return a.secret, nil
	}, jwt.WithValidMethods([]string{jwt.SigningMethodHS256.Alg()}))
	if err != nil && !errors.Is(err, jwt.ErrTokenExpired) {
		// Expired tokens still have verified signatures and claims, so we can
		// ignore here and re-issue at the caller
		return Claims{}, fmt.Errorf("parse token: %w", err)
	}

	return claims, nil
}
