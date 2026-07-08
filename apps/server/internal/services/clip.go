package services

import (
	"context"

	"github.com/jackc/pgx/v5/pgtype"
	"github.com/skkil/clip/server/internal/db"
)

type ClipService struct {
	db *db.Queries
}

func NewClipService(db *db.Queries) *ClipService {
	return &ClipService{db: db}
}

func (s *ClipService) CreateClip(ctx context.Context, userID pgtype.UUID, content string) (db.Clip, error) {
	return s.db.CreateClip(ctx, db.CreateClipParams{
		UserID:  userID,
		Content: content,
	})
}

func (s *ClipService) GetAllClips(ctx context.Context, userID pgtype.UUID) ([]db.Clip, error) {
	return s.db.ListClips(ctx, db.ListClipsParams{
		UserID: userID,
		Limit:  5,
	})
}
