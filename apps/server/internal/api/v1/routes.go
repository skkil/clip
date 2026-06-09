package v1

import (
	"net/http"

	"github.com/skkil/clip/server/internal/db"
	"github.com/skkil/clip/server/internal/handlers"
	"github.com/skkil/clip/server/internal/services"
)

func RegisterRoutes(mux *http.ServeMux, db *db.Queries) {
	clipService := services.NewClipService(db)
	clipHandler := handlers.NewClipHandler(clipService)

	mux.HandleFunc("POST /v1/copy", clipHandler.Copy)
}
