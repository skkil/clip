package rpc

import (
	"context"

	"connectrpc.com/connect"
	v1 "github.com/skkil/clip/proto/v1"
	"github.com/skkil/clip/proto/v1/v1connect"
	"github.com/skkil/clip/server/internal/services"
	"google.golang.org/protobuf/types/known/timestamppb"
)

type ClipHandler struct {
	v1connect.UnimplementedClipServiceHandler
	clipService *services.ClipService
}

func NewClipHandler(clipService *services.ClipService) *ClipHandler {
	return &ClipHandler{clipService: clipService}
}

func (h *ClipHandler) CreateClip(
	ctx context.Context,
	req *connect.Request[v1.CreateClipRequest],
) (*connect.Response[v1.CreateClipResponse], error) {
	clip, err := h.clipService.CreateClip(ctx, req.Msg.Content)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}

	return connect.NewResponse(&v1.CreateClipResponse{
		Id:        clip.ID.String(),
		CreatedAt: timestamppb.New(clip.CreatedAt.Time),
	}), nil
}
