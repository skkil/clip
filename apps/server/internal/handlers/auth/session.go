package auth

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"net/http"
	"os"
	"strings"
	"time"
)

const sessionCookieName = "clip_session"
const sessionTTL = 24 * time.Hour

// Session is a minimal HMAC-signed cookie scoped to the device-approval
// flow, not a general auth-session system. See plan's Open decision #1.
func signingKey() []byte {
	return []byte(os.Getenv("JWT_SIGNING_KEY"))
}

func sign(value string) string {
	mac := hmac.New(sha256.New, signingKey())
	mac.Write([]byte(value))
	return hex.EncodeToString(mac.Sum(nil))
}

// SetSessionCookie sets a signed cookie identifying userID for the
// approval-flow browser session.
func SetSessionCookie(w http.ResponseWriter, userID string) {
	value := base64.RawURLEncoding.EncodeToString([]byte(userID))
	sig := sign(value)
	http.SetCookie(w, &http.Cookie{
		Name:     sessionCookieName,
		Value:    value + "." + sig,
		Path:     "/",
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
		Expires:  time.Now().Add(sessionTTL),
	})
}

// ClearSessionCookie removes the session cookie, used when it refers to a
// user that no longer exists (e.g. after a database reset) so the browser
// stops presenting a session that will only ever fail server-side.
func ClearSessionCookie(w http.ResponseWriter) {
	http.SetCookie(w, &http.Cookie{
		Name:     sessionCookieName,
		Value:    "",
		Path:     "/",
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
		MaxAge:   -1,
	})
}

// UserIDFromRequest returns the signed-in user ID from the session cookie,
// or "" if there is none or it fails verification.
func UserIDFromRequest(r *http.Request) string {
	cookie, err := r.Cookie(sessionCookieName)
	if err != nil {
		return ""
	}

	parts := strings.SplitN(cookie.Value, ".", 2)
	if len(parts) != 2 {
		return ""
	}
	value, sig := parts[0], parts[1]

	if !hmac.Equal([]byte(sig), []byte(sign(value))) {
		return ""
	}

	raw, err := base64.RawURLEncoding.DecodeString(value)
	if err != nil {
		return ""
	}
	return string(raw)
}
