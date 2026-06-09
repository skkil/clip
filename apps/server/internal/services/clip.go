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

func (s *ClipService) CreateClip(ctx context.Context, content string) error {
	_, err := s.db.CreateClip(context.Background(), content)
	return err
}
