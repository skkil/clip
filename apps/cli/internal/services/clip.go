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

func (s *ClipService) client() v1connect.ClipServiceClient {
	auth := NewAuthService(config.ServerURL())
	return v1connect.NewClipServiceClient(
		http.DefaultClient,
		config.ServerURL(),
		connect.WithInterceptors(newAuthInterceptor(auth)),
	)
}

func (s *ClipService) Save(text string) error {
	clip, err := s.client().CreateClip(context.Background(), connect.NewRequest(&v1.CreateClipRequest{
		Content: text,
	}))
	if err != nil {
		return fmt.Errorf("failed to create clip: %w", err)
	}

	fmt.Printf("Clip created with ID: %s\n", clip.Msg.Id)
	return nil
}

func (s *ClipService) List(ctx context.Context) ([]*v1.Clip, error) {
	resp, err := s.client().ListClips(ctx, connect.NewRequest(&v1.ListClipsRequest{}))
	if err != nil {
		return nil, fmt.Errorf("failed to list clips: %w", err)
	}

	return resp.Msg.Clips, nil
}
