package v1

import (
	"net/http"

	"github.com/skkil/clip/proto/v1/v1connect"
	"github.com/skkil/clip/server/internal/db"
	"github.com/skkil/clip/server/internal/handlers"
	"github.com/skkil/clip/server/internal/handlers/auth"
	"github.com/skkil/clip/server/internal/handlers/auth/oauth2"
	"github.com/skkil/clip/server/internal/middleware"
	"github.com/skkil/clip/server/internal/rpc"
	"github.com/skkil/clip/server/internal/services"
)

func RegisterRoutes(mux *http.ServeMux, queries *db.Queries) {
	clipService := services.NewClipService(queries)
	userService := services.NewUserService(queries)
	deviceService := services.NewDeviceService(queries)
	tokenService := services.NewTokenService(queries)

	requireAuth := middleware.Auth(tokenService)

	clip := handlers.NewClipHandler(clipService)
	mux.Handle("POST /v1/clips", requireAuth(http.HandlerFunc(clip.Copy)))
	mux.Handle("GET /v1/clips", requireAuth(http.HandlerFunc(clip.GetAll)))

	authHandler := auth.NewAuthHandler()
	mux.HandleFunc("GET /v1/auth/login", authHandler.LoginPage)

	deviceAuth := auth.NewDeviceAuthHandler(deviceService, tokenService, userService)
	mux.HandleFunc("POST /v1/auth/device/code", deviceAuth.RequestCode)
	mux.HandleFunc("POST /v1/auth/device/token", deviceAuth.Poll)
	mux.HandleFunc("GET /v1/auth/device", deviceAuth.VerifyPage)
	mux.HandleFunc("POST /v1/auth/device/approve", deviceAuth.Approve)
	mux.HandleFunc("POST /v1/auth/token/refresh", deviceAuth.Refresh)

	google := oauth2.NewGoogleHandler(userService)
	mux.HandleFunc("GET /v1/auth/google", google.GoogleLogin)
	mux.HandleFunc("GET /v1/auth/google/callback", google.GoogleCallback)

	path, handler := v1connect.NewClipServiceHandler(rpc.NewClipHandler(clipService))
	mux.Handle(path, requireAuth(handler))
}
