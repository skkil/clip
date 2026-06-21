package services

import (
	"context"

	"github.com/skkil/clip/server/internal/db"
)

type ClipService struct {
	db *db.Queries
}

func NewClipService(db *db.Queries) *ClipService {
	return &ClipService{db: db}
}

func (s *ClipService) CreateClip(ctx context.Context, content string) (db.Clip, error) {
	return s.db.CreateClip(ctx, content)
}

func (s *ClipService) GetAllClips(ctx context.Context) ([]db.Clip, error) {
	return s.db.ListClips(ctx, 5)
}
