package services

import (
	"context"
	"fmt"
	"sync"

	"connectrpc.com/connect"
	"github.com/skkil/clip/cli/internal/config"
)

// authInterceptor attaches a bearer access token to every RPC. The access
// token lives only in process memory (config.LoadRefreshToken reads the
// keyring-stored refresh token to mint one lazily); on a 401 it refreshes
// once and retries the call with the new access token.
type authInterceptor struct {
	auth *AuthService

	mu          sync.Mutex
	accessToken string
}

func newAuthInterceptor(auth *AuthService) *authInterceptor {
	return &authInterceptor{auth: auth}
}

func (i *authInterceptor) ensureAccessToken(ctx context.Context) (string, error) {
	i.mu.Lock()
	defer i.mu.Unlock()

	if i.accessToken != "" {
		return i.accessToken, nil
	}

	refreshToken, err := config.LoadRefreshToken()
	if err != nil || refreshToken == "" {
		return "", fmt.Errorf("not logged in: run `clip auth login` first")
	}

	newRefreshToken, accessToken, err := i.auth.Refresh(ctx, refreshToken)
	if err != nil {
		return "", err
	}
	if err := config.SaveRefreshToken(newRefreshToken); err != nil {
		return "", err
	}

	i.accessToken = accessToken
	return i.accessToken, nil
}

func (i *authInterceptor) forceRefresh(ctx context.Context) (string, error) {
	refreshToken, err := config.LoadRefreshToken()
	if err != nil || refreshToken == "" {
		return "", fmt.Errorf("not logged in: run `clip auth login` first")
	}

	newRefreshToken, accessToken, err := i.auth.Refresh(ctx, refreshToken)
	if err != nil {
		return "", err
	}
	if err := config.SaveRefreshToken(newRefreshToken); err != nil {
		return "", err
	}

	i.mu.Lock()
	i.accessToken = accessToken
	i.mu.Unlock()

	return accessToken, nil
}

func (i *authInterceptor) WrapUnary(next connect.UnaryFunc) connect.UnaryFunc {
	return func(ctx context.Context, req connect.AnyRequest) (connect.AnyResponse, error) {
		accessToken, err := i.ensureAccessToken(ctx)
		if err != nil {
			return nil, err
		}
		req.Header().Set("Authorization", "Bearer "+accessToken)

		resp, err := next(ctx, req)
		if err == nil || connect.CodeOf(err) != connect.CodeUnauthenticated {
			return resp, err
		}

		accessToken, err = i.forceRefresh(ctx)
		if err != nil {
			return nil, err
		}
		req.Header().Set("Authorization", "Bearer "+accessToken)

		return next(ctx, req)
	}
}

func (i *authInterceptor) WrapStreamingClient(next connect.StreamingClientFunc) connect.StreamingClientFunc {
	return next
}

func (i *authInterceptor) WrapStreamingHandler(next connect.StreamingHandlerFunc) connect.StreamingHandlerFunc {
	return next
}
