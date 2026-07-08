package rpc

import (
	"context"

	"connectrpc.com/connect"
	v1 "github.com/skkil/clip/proto/v1"
	"github.com/skkil/clip/proto/v1/v1connect"
	"github.com/skkil/clip/server/internal/middleware"
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
	userID, ok := middleware.UserIDFromContext(ctx)
	if !ok {
		return nil, connect.NewError(connect.CodeUnauthenticated, nil)
	}

	clip, err := h.clipService.CreateClip(ctx, userID, req.Msg.Content)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}

	return connect.NewResponse(&v1.CreateClipResponse{
		Id:        clip.ID.String(),
		CreatedAt: timestamppb.New(clip.CreatedAt.Time),
	}), nil
}

func (h *ClipHandler) ListClips(
	ctx context.Context,
	req *connect.Request[v1.ListClipsRequest],
) (*connect.Response[v1.ListClipsResponse], error) {
	userID, ok := middleware.UserIDFromContext(ctx)
	if !ok {
		return nil, connect.NewError(connect.CodeUnauthenticated, nil)
	}

	clips, err := h.clipService.GetAllClips(ctx, userID)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}

	resp := &v1.ListClipsResponse{
		Clips: make([]*v1.Clip, 0, len(clips)),
	}
	for _, clip := range clips {
		resp.Clips = append(resp.Clips, &v1.Clip{
			Id:        clip.ID.String(),
			Content:   clip.Content,
			CreatedAt: timestamppb.New(clip.CreatedAt.Time),
		})
	}

	return connect.NewResponse(resp), nil
}
