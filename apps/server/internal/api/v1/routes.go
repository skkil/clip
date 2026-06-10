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

	mux.HandleFunc("POST /v1/copy", handlers.NewClipHandler(clipService).Copy)

	path, handler := v1connect.NewClipServiceHandler(rpc.NewClipHandler(clipService))
	mux.Handle(path, handler)
}
