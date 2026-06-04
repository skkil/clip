package handler

import (
	"io"
	"net/http"
)

func Copy(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "failed to read body", http.StatusBadRequest)
		return
	}

	if len(body) == 0 {
		http.Error(w, "content is required", http.StatusBadRequest)
		return
	}

	println("Copied content: ", string(body))
	w.WriteHeader(http.StatusNoContent)
}
