package services

import (
	"context"
	"errors"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgtype"
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

var ErrUserNotFound = errors.New("user not found")

// GetByID looks up a user by ID, used to validate that a signed session
// cookie still refers to a real row (it can outlive the user it names, e.g.
// after a database reset in development).
func (s *UserService) GetByID(ctx context.Context, id pgtype.UUID) (db.User, error) {
	user, err := s.db.GetUserByID(ctx, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return db.User{}, ErrUserNotFound
		}
		return db.User{}, err
	}
	return user, nil
}

func (s *UserService) CreateUser(ctx context.Context, req CreateUserRequest) (CreateUserResponse, error) {
	_, err := s.db.CreateUser(ctx, db.CreateUserParams{
		Name:  pgtype.Text{String: req.Name, Valid: req.Name != ""},
		Email: req.Email,
	})

	return CreateUserResponse{}, err
}

// GetOrCreateByOAuthIdentity looks up the user linked to a (provider,
// provider_user_id) pair, creating both the user and the oauth_identities
// link on first login for that identity.
func (s *UserService) GetOrCreateByOAuthIdentity(ctx context.Context, provider db.OauthProvider, providerUserID, name, email string) (db.User, error) {
	identity, err := s.db.GetOAuthIdentity(ctx, db.GetOAuthIdentityParams{
		Provider:       provider,
		ProviderUserID: providerUserID,
	})
	if err == nil {
		return s.db.GetUserByID(ctx, identity.UserID)
	}
	if !errors.Is(err, pgx.ErrNoRows) {
		return db.User{}, err
	}

	nameArg := pgtype.Text{String: name, Valid: name != ""}
	row, err := s.db.CreateUserWithOAuthIdentity(ctx, db.CreateUserWithOAuthIdentityParams{
		Name:           nameArg,
		Email:          email,
		Provider:       provider,
		ProviderUserID: providerUserID,
	})
	if err != nil {
		return db.User{}, err
	}

	return db.User{
		ID:        row.UserID,
		Name:      row.Name,
		Email:     row.Email,
		CreatedAt: row.CreatedAt,
	}, nil
}
