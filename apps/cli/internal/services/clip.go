package services

import (
	"context"
	"fmt"
	"log"
	"net/http"

	"connectrpc.com/connect"
	v1 "github.com/skkil/clip/proto/v1"
	"github.com/skkil/clip/proto/v1/v1connect"
)

type ClipService struct{}

func NewClipService() *ClipService {
	return &ClipService{}
}

func (s *ClipService) Save(text string) error {
	client := v1connect.NewClipServiceClient(http.DefaultClient, "http://localhost:8080")

	clip, err := client.CreateClip(context.Background(), connect.NewRequest(&v1.CreateClipRequest{
		Content: text,
	}))
	if err != nil {
		log.Fatalf("fail to create clip: %v", err)
	}

	fmt.Printf("Clip created with ID: %s\n", clip.Msg.Id)
	return nil
}
