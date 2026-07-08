package config

import "github.com/zalando/go-keyring"

// serverURL is baked into the binary at build time; override only via
// -ldflags "-X github.com/skkil/clip/cli/internal/config.serverURL=..."
// so an installed CLI can't be silently redirected to a different host.
var serverURL = "http://localhost:8080"

const keyringService = "clip-cli"

func ServerURL() string {
	return serverURL
}

func keyringAccount() string {
	return serverURL
}

func LoadRefreshToken() (string, error) {
	return keyring.Get(keyringService, keyringAccount())
}

func SaveRefreshToken(token string) error {
	return keyring.Set(keyringService, keyringAccount(), token)
}

func DeleteRefreshToken() error {
	return keyring.Delete(keyringService, keyringAccount())
}
