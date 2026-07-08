package handlers

import (
	"encoding/json"
	"io"
	"log"
	"net/http"

	"github.com/skkil/clip/server/internal/middleware"
	"github.com/skkil/clip/server/internal/services"
)

type ClipHandler struct {
	clipService *services.ClipService
}

func NewClipHandler(clipService *services.ClipService) *ClipHandler {
	return &ClipHandler{clipService: clipService}
}

func (h *ClipHandler) Copy(w http.ResponseWriter, r *http.Request) {
	userID, ok := middleware.UserIDFromContext(r.Context())
	if !ok {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}

	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "failed to read body", http.StatusBadRequest)
		return
	}

	if len(body) == 0 {
		http.Error(w, "content is required", http.StatusBadRequest)
		return
	}

	_, err = h.clipService.CreateClip(r.Context(), userID, string(body))
	if err != nil {
		log.Printf("failed to create clip: %v\n", err)
		http.Error(w, "failed to save clip", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *ClipHandler) GetAll(w http.ResponseWriter, r *http.Request) {
	userID, ok := middleware.UserIDFromContext(r.Context())
	if !ok {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}

	clips, err := h.clipService.GetAllClips(r.Context(), userID)
	if err != nil {
		log.Printf("failed to get clips: %v\n", err)
		http.Error(w, "failed to retrieve clips", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusOK)
	if err := json.NewEncoder(w).Encode(clips); err != nil {
		http.Error(w, "failed to encode response", http.StatusInternalServerError)
		return
	}
}
