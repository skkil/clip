package services

import (
	"context"
	"crypto/rand"
	"errors"
	"time"

	"github.com/jackc/pgx/v5/pgtype"
	"github.com/skkil/clip/server/internal/db"
)

// DeviceStatus re-exports the sqlc-generated enum type so callers outside
// this package don't need to import internal/db directly for status checks.
type DeviceStatus = db.DeviceStatus

const (
	DeviceStatusPending  = db.DeviceStatusPending
	DeviceStatusApproved = db.DeviceStatusApproved
	DeviceStatusDenied   = db.DeviceStatusDenied
	DeviceStatusExpired  = db.DeviceStatusExpired
)

var ErrDeviceNotFound = errors.New("device not found")

// userCodeAlphabet avoids visually ambiguous characters (0/O, 1/I).
const userCodeAlphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"

type DeviceService struct {
	db *db.Queries
}

func NewDeviceService(db *db.Queries) *DeviceService {
	return &DeviceService{db: db}
}

type DeviceCode struct {
	DeviceCode string
	UserCode   string
	ExpiresIn  int64
}

func generateUserCode() (string, error) {
	buf := make([]byte, 8)
	if _, err := rand.Read(buf); err != nil {
		return "", err
	}
	code := make([]byte, 8)
	for i, b := range buf {
		code[i] = userCodeAlphabet[int(b)%len(userCodeAlphabet)]
	}
	return string(code[:4]) + "-" + string(code[4:]), nil
}

// CreateDevice generates a device_code/user_code pair and inserts the
// pending devices row. The raw device_code is returned exactly once here;
// only its hash is persisted.
func (s *DeviceService) CreateDevice(ctx context.Context) (DeviceCode, error) {
	rawDeviceCode, err := generateToken()
	if err != nil {
		return DeviceCode{}, err
	}

	userCode, err := generateUserCode()
	if err != nil {
		return DeviceCode{}, err
	}

	device, err := s.db.CreateDevice(ctx, db.CreateDeviceParams{
		DeviceCodeHash: hashToken(rawDeviceCode),
		UserCode:       userCode,
		ExpiresAt:      pgtype.Timestamptz{Time: time.Now().Add(DeviceCodeTTL), Valid: true},
	})
	if err != nil {
		return DeviceCode{}, err
	}

	return DeviceCode{
		DeviceCode: rawDeviceCode,
		UserCode:   device.UserCode,
		ExpiresIn:  int64(DeviceCodeTTL.Seconds()),
	}, nil
}

func (s *DeviceService) GetByDeviceCode(ctx context.Context, rawDeviceCode string) (db.Device, error) {
	device, err := s.db.GetDeviceByDeviceCodeHash(ctx, hashToken(rawDeviceCode))
	if err != nil {
		return db.Device{}, ErrDeviceNotFound
	}
	return device, nil
}

func (s *DeviceService) GetByUserCode(ctx context.Context, userCode string) (db.Device, error) {
	device, err := s.db.GetDeviceByUserCode(ctx, userCode)
	if err != nil {
		return db.Device{}, ErrDeviceNotFound
	}
	return device, nil
}

func (s *DeviceService) Approve(ctx context.Context, deviceID, userID pgtype.UUID) (db.Device, error) {
	return s.db.ApproveDevice(ctx, db.ApproveDeviceParams{
		ID:     deviceID,
		UserID: userID,
	})
}

func IsDeviceCodeExpired(device db.Device) bool {
	return device.ExpiresAt.Valid && time.Now().After(device.ExpiresAt.Time)
}
