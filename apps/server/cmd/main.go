package main

import (
	"context"
	"log"
	"net/http"
	"os"

	"github.com/jackc/pgx/v5/pgxpool"
	_ "github.com/joho/godotenv/autoload"
	v1 "github.com/skkil/clip/server/internal/api/v1"
	"github.com/skkil/clip/server/internal/db"
	"github.com/skkil/clip/server/internal/middleware"
)

func main() {
	pool, err := pgxpool.New(context.Background(), os.Getenv("DATABASE_URL"))
	if err != nil {
		log.Fatal("failed to connect to database: ", err)
	}
	defer pool.Close()

	queries := db.New(pool)
	mux := http.NewServeMux()

	v1.RegisterRoutes(mux, queries)

	handler := middleware.Chain(middleware.LogRequest)(mux)

	log.Println("listening on :8080")
	if err := http.ListenAndServe(":8080", handler); err != nil {
		log.Fatal(err)
	}
}
