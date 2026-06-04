package main

import (
	"log"
	"net/http"

	v1 "github.com/skkil/clip/server/internal/handler/v1"
)

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("POST /v1/copy", v1.Copy)

	log.Println("listening on :8080")
	if err := http.ListenAndServe(":8080", mux); err != nil {
		log.Fatal(err)
	}
}
