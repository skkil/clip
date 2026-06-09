package handlers

import (
	"io"
	"log"
	"net/http"

	"github.com/skkil/clip/server/internal/services"
)

type ClipHandler struct {
	clipService *services.ClipService
}

func NewClipHandler(clipService *services.ClipService) *ClipHandler {
	return &ClipHandler{clipService: clipService}
}

func (h *ClipHandler) Copy(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "failed to read body", http.StatusBadRequest)
		return
	}

	if len(body) == 0 {
		http.Error(w, "content is required", http.StatusBadRequest)
		return
	}

	err = h.clipService.CreateClip(r.Context(), string(body))
	if err != nil {
		log.Printf("failed to create clip: %v\n", err)
		http.Error(w, "failed to save clip", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}
