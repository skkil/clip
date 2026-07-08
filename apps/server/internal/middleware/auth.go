package middleware

import (
	"context"
	"net/http"
	"strings"

	"github.com/jackc/pgx/v5/pgtype"
	"github.com/skkil/clip/server/internal/services"
)

type contextKey string

const (
	userIDContextKey   contextKey = "user_id"
	deviceIDContextKey contextKey = "device_id"
)

func UserIDFromContext(ctx context.Context) (pgtype.UUID, bool) {
	id, ok := ctx.Value(userIDContextKey).(pgtype.UUID)
	return id, ok
}

func DeviceIDFromContext(ctx context.Context) (pgtype.UUID, bool) {
	id, ok := ctx.Value(deviceIDContextKey).(pgtype.UUID)
	return id, ok
}

// Auth validates the bearer access token on every request and attaches the
// resolved user/device IDs to the request context. Requests without a
// valid, unexpired access token are rejected with 401.
func Auth(tokens *services.TokenService) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authHeader := r.Header.Get("Authorization")
			rawToken, ok := strings.CutPrefix(authHeader, "Bearer ")
			if !ok || rawToken == "" {
				http.Error(w, "unauthorized", http.StatusUnauthorized)
				return
			}

			userID, deviceID, err := tokens.ValidateAccessToken(r.Context(), rawToken)
			if err != nil {
				http.Error(w, "unauthorized", http.StatusUnauthorized)
				return
			}

			ctx := context.WithValue(r.Context(), userIDContextKey, userID)
			ctx = context.WithValue(ctx, deviceIDContextKey, deviceID)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}
