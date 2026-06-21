package v1

import (
	"net/http"

	"github.com/skkil/clip/proto/v1/v1connect"
	"github.com/skkil/clip/server/internal/db"
	"github.com/skkil/clip/server/internal/handlers"
	"github.com/skkil/clip/server/internal/rpc"
	"github.com/skkil/clip/server/internal/services"
)

func RegisterRoutes(mux *http.ServeMux, db *db.Queries) {
	clipService := services.NewClipService(db)
	clip := handlers.NewClipHandler(clipService)
	mux.HandleFunc("POST /v1/clips", clip.Copy)
	mux.HandleFunc("GET /v1/clips", clip.GetAll)

	auth := handlers.NewAuthHandler()
	mux.HandleFunc("GET /v1/auth/login", auth.LoginPage)

	path, handler := v1connect.NewClipServiceHandler(rpc.NewClipHandler(clipService))
	mux.Handle(path, handler)
}
