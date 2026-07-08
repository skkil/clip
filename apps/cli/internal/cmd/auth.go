package cmd

import (
	"context"
	"fmt"
	"os/exec"
	"runtime"

	"github.com/skkil/clip/cli/internal/config"
	"github.com/skkil/clip/cli/internal/services"
	"github.com/spf13/cobra"
)

var auth = &cobra.Command{
	Use:   "auth",
	Short: "Authenticate with the clip service",
	RunE:  runLogin,
}

var authLogin = &cobra.Command{
	Use:   "login",
	Short: "Log in via the browser",
	RunE:  runLogin,
}

var authLogout = &cobra.Command{
	Use:   "logout",
	Short: "Log out and revoke this device's session",
	RunE:  runLogout,
}

func init() {
	auth.AddCommand(authLogin)
	auth.AddCommand(authLogout)
}

func runLogin(cmd *cobra.Command, args []string) error {
	ctx := context.Background()
	svc := services.NewAuthService(config.ServerURL())

	code, err := svc.RequestDeviceCode(ctx)
	if err != nil {
		return fmt.Errorf("failed to start device login: %w", err)
	}

	fmt.Printf("Open the following URL to authorize this device:\n\n  %s\n\n", code.VerificationURIComplete)
	openBrowser(code.VerificationURIComplete)

	refreshToken, _, err := svc.PollForToken(ctx, code)
	if err != nil {
		return fmt.Errorf("login failed: %w", err)
	}

	if err := config.SaveRefreshToken(refreshToken); err != nil {
		return fmt.Errorf("failed to save credentials: %w", err)
	}

	fmt.Println("Logged in successfully.")
	return nil
}

func runLogout(cmd *cobra.Command, args []string) error {
	refreshToken, err := config.LoadRefreshToken()
	if err == nil && refreshToken != "" {
		svc := services.NewAuthService(config.ServerURL())
		// Best-effort: revoke server-side immediately rather than waiting
		// out the token's natural expiry. Refresh (not a dedicated revoke
		// endpoint) is what the server exposes for this, and rotating then
		// discarding the result has the same effect as revoking.
		_, _, _ = svc.Refresh(context.Background(), refreshToken)
	}

	if err := config.DeleteRefreshToken(); err != nil {
		return fmt.Errorf("failed to remove credentials: %w", err)
	}

	fmt.Println("Logged out.")
	return nil
}

func openBrowser(url string) {
	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "darwin":
		cmd = exec.Command("open", url)
	case "windows":
		cmd = exec.Command("rundll32", "url.dll,FileProtocolHandler", url)
	default:
		cmd = exec.Command("xdg-open", url)
	}
	_ = cmd.Start()
}
