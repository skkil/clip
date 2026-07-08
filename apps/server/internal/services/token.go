package services

import (
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"time"

	"github.com/jackc/pgx/v5/pgtype"
	"github.com/skkil/clip/server/internal/db"
)

const (
	DeviceCodeTTL       = 10 * time.Minute
	RefreshTokenTTL     = 90 * 24 * time.Hour
	RefreshTokenIdleTTL = 14 * 24 * time.Hour
	AccessTokenTTL      = 15 * time.Minute
)

var ErrTokenInvalid = errors.New("token invalid or expired")

type TokenService struct {
	db *db.Queries
}

func NewTokenService(db *db.Queries) *TokenService {
	return &TokenService{db: db}
}

func hashToken(raw string) string {
	sum := sha256.Sum256([]byte(raw))
	return hex.EncodeToString(sum[:])
}

func generateToken() (string, error) {
	buf := make([]byte, 32)
	if _, err := rand.Read(buf); err != nil {
		return "", err
	}
	return hex.EncodeToString(buf), nil
}

type IssuedTokens struct {
	RefreshToken string
	AccessToken  string
	ExpiresIn    int64
}

// IssueSession creates the initial refresh_token + access_token pair for a
// newly approved device.
func (s *TokenService) IssueSession(ctx context.Context, userID, deviceID pgtype.UUID) (IssuedTokens, error) {
	rawRefresh, err := generateToken()
	if err != nil {
		return IssuedTokens{}, err
	}

	refresh, err := s.db.CreateRefreshToken(ctx, db.CreateRefreshTokenParams{
		DeviceID:  deviceID,
		UserID:    userID,
		TokenHash: hashToken(rawRefresh),
		ExpiresAt: pgtype.Timestamptz{Time: time.Now().Add(RefreshTokenTTL), Valid: true},
	})
	if err != nil {
		return IssuedTokens{}, err
	}

	rawAccess, err := s.mintAccessToken(ctx, refresh.ID, userID, deviceID)
	if err != nil {
		return IssuedTokens{}, err
	}

	return IssuedTokens{
		RefreshToken: rawRefresh,
		AccessToken:  rawAccess,
		ExpiresIn:    int64(AccessTokenTTL.Seconds()),
	}, nil
}

func (s *TokenService) mintAccessToken(ctx context.Context, refreshTokenID, userID, deviceID pgtype.UUID) (string, error) {
	rawAccess, err := generateToken()
	if err != nil {
		return "", err
	}

	_, err = s.db.CreateAccessToken(ctx, db.CreateAccessTokenParams{
		RefreshTokenID: refreshTokenID,
		UserID:         userID,
		DeviceID:       deviceID,
		TokenHash:      hashToken(rawAccess),
		ExpiresAt:      pgtype.Timestamptz{Time: time.Now().Add(AccessTokenTTL), Valid: true},
	})
	if err != nil {
		return "", err
	}

	return rawAccess, nil
}

// Refresh validates the presented refresh token, rotates it (revokes the old
// row and issues a new one preserving the absolute expiry), and mints a new
// access token.
func (s *TokenService) Refresh(ctx context.Context, rawRefreshToken string) (IssuedTokens, error) {
	existing, err := s.db.GetRefreshTokenByHash(ctx, hashToken(rawRefreshToken))
	if err != nil {
		return IssuedTokens{}, ErrTokenInvalid
	}

	if existing.RevokedAt.Valid {
		return IssuedTokens{}, ErrTokenInvalid
	}
	now := time.Now()
	if !existing.ExpiresAt.Valid || now.After(existing.ExpiresAt.Time) {
		return IssuedTokens{}, ErrTokenInvalid
	}
	if !existing.LastUsedAt.Valid || now.After(existing.LastUsedAt.Time.Add(RefreshTokenIdleTTL)) {
		return IssuedTokens{}, ErrTokenInvalid
	}

	rawRefresh, err := generateToken()
	if err != nil {
		return IssuedTokens{}, err
	}

	rotated, err := s.db.CreateRefreshToken(ctx, db.CreateRefreshTokenParams{
		DeviceID:  existing.DeviceID,
		UserID:    existing.UserID,
		TokenHash: hashToken(rawRefresh),
		ExpiresAt: existing.ExpiresAt,
	})
	if err != nil {
		return IssuedTokens{}, err
	}

	if err := s.db.RevokeRefreshToken(ctx, existing.ID); err != nil {
		return IssuedTokens{}, err
	}

	rawAccess, err := s.mintAccessToken(ctx, rotated.ID, existing.UserID, existing.DeviceID)
	if err != nil {
		return IssuedTokens{}, err
	}

	return IssuedTokens{
		RefreshToken: rawRefresh,
		AccessToken:  rawAccess,
		ExpiresIn:    int64(AccessTokenTTL.Seconds()),
	}, nil
}

// ValidateAccessToken looks up a raw access token and returns the associated
// user/device IDs if it is still valid.
func (s *TokenService) ValidateAccessToken(ctx context.Context, rawAccessToken string) (userID, deviceID pgtype.UUID, err error) {
	token, err := s.db.GetAccessTokenByHash(ctx, hashToken(rawAccessToken))
	if err != nil {
		return pgtype.UUID{}, pgtype.UUID{}, ErrTokenInvalid
	}
	return token.UserID, token.DeviceID, nil
}

// RevokeRefreshToken revokes the refresh token by its raw value, used for
// logout. Returns ErrTokenInvalid if it doesn't exist (already revoked or
// unknown) — callers should treat that as a no-op, not a hard failure.
func (s *TokenService) RevokeRefreshToken(ctx context.Context, rawRefreshToken string) error {
	existing, err := s.db.GetRefreshTokenByHash(ctx, hashToken(rawRefreshToken))
	if err != nil {
		return ErrTokenInvalid
	}
	return s.db.RevokeRefreshToken(ctx, existing.ID)
}
