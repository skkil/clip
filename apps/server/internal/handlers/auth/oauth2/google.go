package oauth2

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"net/http"
	"os"

	"github.com/jackc/pgx/v5/pgtype"
	"github.com/skkil/clip/server/internal/db"
	authhandlers "github.com/skkil/clip/server/internal/handlers/auth"
	"github.com/skkil/clip/server/internal/services"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
)

const userinfoURL = "https://www.googleapis.com/oauth2/v3/userinfo"

type GoogleHandler struct {
	users *services.UserService
	oauth *oauth2.Config
}

func NewGoogleHandler(users *services.UserService) *GoogleHandler {
	return &GoogleHandler{
		users: users,
		oauth: &oauth2.Config{
			ClientID:     os.Getenv("GOOGLE_CLIENT_ID"),
			ClientSecret: os.Getenv("GOOGLE_CLIENT_SECRET"),
			RedirectURL:  os.Getenv("GOOGLE_REDIRECT_URL"),
			Scopes:       []string{"openid", "profile", "email"},
			Endpoint:     google.Endpoint,
		},
	}
}

func stateSigningKey() []byte {
	return []byte(os.Getenv("STATE_SIGNING_KEY"))
}

// signState binds the OAuth state param to the user_code it carries so a
// callback can't be replayed with a different device's user_code.
func signState(userCode string) string {
	mac := hmac.New(sha256.New, stateSigningKey())
	mac.Write([]byte(userCode))
	return userCode + "." + hex.EncodeToString(mac.Sum(nil))
}

func verifyState(state string) (userCode string, ok bool) {
	idx := len(state) - 64 - 1 // hex sha256 = 64 chars, plus '.'
	if idx <= 0 || state[idx] != '.' {
		return "", false
	}
	userCode, sig := state[:idx], state[idx+1:]

	mac := hmac.New(sha256.New, stateSigningKey())
	mac.Write([]byte(userCode))
	expected := hex.EncodeToString(mac.Sum(nil))
	if !hmac.Equal([]byte(sig), []byte(expected)) {
		return "", false
	}
	return userCode, true
}

// GoogleLogin handles GET /v1/auth/google.
func (h *GoogleHandler) GoogleLogin(w http.ResponseWriter, r *http.Request) {
	userCode := r.URL.Query().Get("state")
	url := h.oauth.AuthCodeURL(signState(userCode))
	http.Redirect(w, r, url, http.StatusFound)
}

type googleUserinfo struct {
	Sub   string `json:"sub"`
	Email string `json:"email"`
	Name  string `json:"name"`
}

// GoogleCallback handles GET /v1/auth/google/callback.
func (h *GoogleHandler) GoogleCallback(w http.ResponseWriter, r *http.Request) {
	userCode, ok := verifyState(r.URL.Query().Get("state"))
	if !ok {
		http.Error(w, "invalid state", http.StatusBadRequest)
		return
	}

	code := r.URL.Query().Get("code")
	token, err := h.oauth.Exchange(r.Context(), code)
	if err != nil {
		http.Error(w, "failed to exchange code", http.StatusBadGateway)
		return
	}

	client := h.oauth.Client(r.Context(), token)
	resp, err := client.Get(userinfoURL)
	if err != nil {
		http.Error(w, "failed to fetch userinfo", http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	var info googleUserinfo
	if err := json.NewDecoder(resp.Body).Decode(&info); err != nil {
		http.Error(w, "failed to parse userinfo", http.StatusBadGateway)
		return
	}

	user, err := h.users.GetOrCreateByOAuthIdentity(r.Context(), db.OauthProviderGoogle, info.Sub, info.Name, info.Email)
	if err != nil {
		http.Error(w, "failed to look up user", http.StatusInternalServerError)
		return
	}

	authhandlers.SetSessionCookie(w, uuidString(user.ID))

	redirect := "/v1/auth/device"
	if userCode != "" {
		redirect += "?user_code=" + userCode
	}
	http.Redirect(w, r, redirect, http.StatusFound)
}

func uuidString(id pgtype.UUID) string {
	v, _ := id.Value()
	s, _ := v.(string)
	return s
}
