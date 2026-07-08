package services

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type AuthService struct {
	serverURL string
	client    *http.Client
}

func NewAuthService(serverURL string) *AuthService {
	return &AuthService{serverURL: serverURL, client: http.DefaultClient}
}

type DeviceCode struct {
	DeviceCode              string `json:"device_code"`
	UserCode                string `json:"user_code"`
	VerificationURI         string `json:"verification_uri"`
	VerificationURIComplete string `json:"verification_uri_complete"`
	ExpiresIn               int64  `json:"expires_in"`
	Interval                int64  `json:"interval"`
}

func (s *AuthService) postJSON(ctx context.Context, path string, body, out any) (int, error) {
	buf, err := json.Marshal(body)
	if err != nil {
		return 0, err
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, s.serverURL+path, bytes.NewReader(buf))
	if err != nil {
		return 0, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := s.client.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	if out != nil {
		if err := json.NewDecoder(resp.Body).Decode(out); err != nil {
			return resp.StatusCode, err
		}
	}

	return resp.StatusCode, nil
}

func (s *AuthService) RequestDeviceCode(ctx context.Context) (*DeviceCode, error) {
	var code DeviceCode
	status, err := s.postJSON(ctx, "/v1/auth/device/code", nil, &code)
	if err != nil {
		return nil, err
	}
	if status != http.StatusOK {
		return nil, fmt.Errorf("failed to request device code: status %d", status)
	}
	return &code, nil
}

type tokenResponse struct {
	RefreshToken string `json:"refresh_token"`
	AccessToken  string `json:"access_token"`
	Error        string `json:"error"`
}

// PollForToken polls the device token endpoint, respecting the
// server-returned interval, until the device is approved, denied, or the
// code expires.
func (s *AuthService) PollForToken(ctx context.Context, code *DeviceCode) (refreshToken, accessToken string, err error) {
	interval := time.Duration(code.Interval) * time.Second
	if interval <= 0 {
		interval = 5 * time.Second
	}
	deadline := time.Now().Add(time.Duration(code.ExpiresIn) * time.Second)

	for {
		if time.Now().After(deadline) {
			return "", "", fmt.Errorf("device code expired")
		}

		var resp tokenResponse
		status, err := s.postJSON(ctx, "/v1/auth/device/token", map[string]string{"device_code": code.DeviceCode}, &resp)
		if err != nil {
			return "", "", err
		}

		switch {
		case status == http.StatusOK:
			return resp.RefreshToken, resp.AccessToken, nil
		case resp.Error == "authorization_pending":
			// fall through to wait and retry
		case resp.Error == "access_denied":
			return "", "", fmt.Errorf("authorization denied")
		default:
			return "", "", fmt.Errorf("device authorization failed: %s", resp.Error)
		}

		select {
		case <-ctx.Done():
			return "", "", ctx.Err()
		case <-time.After(interval):
		}
	}
}

// Refresh exchanges a refresh token for a new refresh/access token pair.
func (s *AuthService) Refresh(ctx context.Context, refreshToken string) (newRefreshToken, accessToken string, err error) {
	var resp tokenResponse
	status, err := s.postJSON(ctx, "/v1/auth/token/refresh", map[string]string{"refresh_token": refreshToken}, &resp)
	if err != nil {
		return "", "", err
	}
	if status != http.StatusOK {
		return "", "", fmt.Errorf("failed to refresh token: %s", resp.Error)
	}
	return resp.RefreshToken, resp.AccessToken, nil
}
