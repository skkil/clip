package services

import (
	"context"
	"fmt"
	"net/http"

	"connectrpc.com/connect"
	v1 "github.com/skkil/clip/proto/v1"
	"github.com/skkil/clip/proto/v1/v1connect"

	"github.com/skkil/clip/cli/internal/config"
)

type ClipService struct{}

func NewClipService() *ClipService {
	return &ClipService{}
}

func (s *ClipService) Save(text string) error {
	auth := NewAuthService(config.ServerURL())
	client := v1connect.NewClipServiceClient(
		http.DefaultClient,
		config.ServerURL(),
		connect.WithInterceptors(newAuthInterceptor(auth)),
	)

	clip, err := client.CreateClip(context.Background(), connect.NewRequest(&v1.CreateClipRequest{
		Content: text,
	}))
	if err != nil {
		return fmt.Errorf("failed to create clip: %w", err)
	}

	fmt.Printf("Clip created with ID: %s\n", clip.Msg.Id)
	return nil
}
