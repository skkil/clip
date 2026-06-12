package services

import (
	"context"

	"github.com/skkil/clip/server/internal/db"
)

type UserService struct {
	db *db.Queries
}

func NewUserService(db *db.Queries) *UserService {
	return &UserService{db: db}
}

type CreateUserRequest struct {
	Name  string
	Email string
}

type CreateUserResponse struct{}

func (s *UserService) CreateUser(ctx context.Context, req CreateUserRequest) (CreateUserResponse, error) {
	_, err := s.db.CreateUser(ctx, db.CreateUserParams{
		Name:  req.Name,
		Email: req.Email,
	})

	return CreateUserResponse{}, err
}
