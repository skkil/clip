package auth

import (
	"encoding/json"
	"errors"
	"html/template"
	"net/http"
	"sync"

	"github.com/jackc/pgx/v5/pgtype"
	"github.com/skkil/clip/server/internal/services"
)

var deviceVerifyPage *template.Template

func init() {
	deviceVerifyPage = template.Must(template.ParseFiles("templates/v1/auth/device/index.html"))
}

// pendingDeviceTokens relays the raw refresh/access token pair minted at
// Approve time to the CLI's next Poll call. Only hashes are ever persisted
// to the DB, so the raw values have nowhere to live between the two HTTP
// requests except this short-lived, in-memory handoff — consumed (read and
// deleted) the moment Poll observes the device as approved.
type pendingDeviceTokens struct {
	mu     sync.Mutex
	tokens map[string]services.IssuedTokens
}

func newPendingDeviceTokens() *pendingDeviceTokens {
	return &pendingDeviceTokens{tokens: make(map[string]services.IssuedTokens)}
}

func (p *pendingDeviceTokens) put(deviceID string, tokens services.IssuedTokens) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.tokens[deviceID] = tokens
}

func (p *pendingDeviceTokens) take(deviceID string) (services.IssuedTokens, bool) {
	p.mu.Lock()
	defer p.mu.Unlock()
	tokens, ok := p.tokens[deviceID]
	if ok {
		delete(p.tokens, deviceID)
	}
	return tokens, ok
}

type DeviceAuthHandler struct {
	devices *services.DeviceService
	tokens  *services.TokenService
	users   *services.UserService
	pending *pendingDeviceTokens
}

func NewDeviceAuthHandler(devices *services.DeviceService, tokens *services.TokenService, users *services.UserService) *DeviceAuthHandler {
	return &DeviceAuthHandler{
		devices: devices,
		tokens:  tokens,
		users:   users,
		pending: newPendingDeviceTokens(),
	}
}

// sessionUserID validates that the cookie's user ID still refers to a real
// user, clearing the cookie if it doesn't (e.g. the database was reset out
// from under a browser that still holds an old, signature-valid cookie).
func (h *DeviceAuthHandler) sessionUserID(w http.ResponseWriter, r *http.Request) (pgtype.UUID, bool) {
	rawUserID := UserIDFromRequest(r)
	if rawUserID == "" {
		return pgtype.UUID{}, false
	}

	var userID pgtype.UUID
	if err := userID.Scan(rawUserID); err != nil {
		ClearSessionCookie(w)
		return pgtype.UUID{}, false
	}

	if _, err := h.users.GetByID(r.Context(), userID); err != nil {
		ClearSessionCookie(w)
		return pgtype.UUID{}, false
	}

	return userID, true
}

func writeJSON(w http.ResponseWriter, status int, body any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(body)
}

func writeError(w http.ResponseWriter, status int, code string) {
	writeJSON(w, status, map[string]string{"error": code})
}

func requestBaseURL(r *http.Request) string {
	scheme := "https"
	if r.TLS == nil {
		scheme = "http"
	}
	return scheme + "://" + r.Host
}

// RequestCode handles POST /v1/auth/device/code.
func (h *DeviceAuthHandler) RequestCode(w http.ResponseWriter, r *http.Request) {
	code, err := h.devices.CreateDevice(r.Context())
	if err != nil {
		http.Error(w, "failed to create device code", http.StatusInternalServerError)
		return
	}

	base := requestBaseURL(r)
	writeJSON(w, http.StatusOK, map[string]any{
		"device_code":               code.DeviceCode,
		"user_code":                 code.UserCode,
		"verification_uri":          base + "/v1/auth/device",
		"verification_uri_complete": base + "/v1/auth/device?user_code=" + code.UserCode,
		"expires_in":                code.ExpiresIn,
		"interval":                  5,
	})
}

type pollRequest struct {
	DeviceCode string `json:"device_code"`
}

// Poll handles POST /v1/auth/device/token.
func (h *DeviceAuthHandler) Poll(w http.ResponseWriter, r *http.Request) {
	var req pollRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid_request")
		return
	}

	device, err := h.devices.GetByDeviceCode(r.Context(), req.DeviceCode)
	if err != nil {
		writeError(w, http.StatusBadRequest, "expired_token")
		return
	}

	switch device.Status {
	case services.DeviceStatusPending:
		if services.IsDeviceCodeExpired(device) {
			writeError(w, http.StatusBadRequest, "expired_token")
			return
		}
		writeError(w, http.StatusAccepted, "authorization_pending")
	case services.DeviceStatusApproved:
		deviceID := uuidString(device.ID)
		tokens, ok := h.pending.take(deviceID)
		if !ok {
			// Already claimed by an earlier poll, or the process restarted
			// mid-flow; the CLI must re-run the device flow from scratch.
			writeError(w, http.StatusBadRequest, "expired_token")
			return
		}
		writeJSON(w, http.StatusOK, map[string]any{
			"refresh_token": tokens.RefreshToken,
			"access_token":  tokens.AccessToken,
			"expires_in":    tokens.ExpiresIn,
		})
	case services.DeviceStatusDenied:
		writeError(w, http.StatusBadRequest, "access_denied")
	default:
		writeError(w, http.StatusBadRequest, "expired_token")
	}
}

// VerifyPage handles GET /v1/auth/device.
func (h *DeviceAuthHandler) VerifyPage(w http.ResponseWriter, r *http.Request) {
	userCode := r.URL.Query().Get("user_code")

	if _, ok := h.sessionUserID(w, r); !ok {
		http.Redirect(w, r, "/v1/auth/google?state="+userCode, http.StatusFound)
		return
	}

	if err := deviceVerifyPage.Execute(w, map[string]string{"UserCode": userCode}); err != nil {
		http.Error(w, "failed to render verification page", http.StatusInternalServerError)
	}
}

type approveRequest struct {
	UserCode string `json:"user_code"`
}

// Approve handles POST /v1/auth/device/approve.
func (h *DeviceAuthHandler) Approve(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.sessionUserID(w, r)
	if !ok {
		http.Error(w, "not signed in", http.StatusUnauthorized)
		return
	}

	var req approveRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid_request")
		return
	}

	device, err := h.devices.GetByUserCode(r.Context(), req.UserCode)
	if err != nil {
		writeError(w, http.StatusBadRequest, "expired_token")
		return
	}
	if device.Status != services.DeviceStatusPending || services.IsDeviceCodeExpired(device) {
		writeError(w, http.StatusBadRequest, "expired_token")
		return
	}

	approved, err := h.devices.Approve(r.Context(), device.ID, userID)
	if err != nil {
		http.Error(w, "failed to approve device", http.StatusInternalServerError)
		return
	}

	tokens, err := h.tokens.IssueSession(r.Context(), userID, approved.ID)
	if err != nil {
		http.Error(w, "failed to issue session", http.StatusInternalServerError)
		return
	}
	h.pending.put(uuidString(approved.ID), tokens)

	writeJSON(w, http.StatusOK, map[string]string{"status": "approved"})
}

type refreshRequest struct {
	RefreshToken string `json:"refresh_token"`
}

// Refresh handles POST /v1/auth/token/refresh.
func (h *DeviceAuthHandler) Refresh(w http.ResponseWriter, r *http.Request) {
	var req refreshRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid_request")
		return
	}

	tokens, err := h.tokens.Refresh(r.Context(), req.RefreshToken)
	if err != nil {
		if errors.Is(err, services.ErrTokenInvalid) {
			writeError(w, http.StatusUnauthorized, "invalid_grant")
			return
		}
		http.Error(w, "failed to refresh token", http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"refresh_token": tokens.RefreshToken,
		"access_token":  tokens.AccessToken,
		"expires_in":    tokens.ExpiresIn,
	})
}

func uuidString(id pgtype.UUID) string {
	b, _ := id.Value()
	s, _ := b.(string)
	return s
}
