package main

import (
	"encoding/json"
	"log"
	"net/http"
)

type healthResponse struct {
	Status string `json:"status"`
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	response := healthResponse{
		Status: "ok",
	}

	if err := json.NewEncoder(w).Encode(response); err != nil {
		http.Error(w, "failed to encode response", http.StatusInternalServerError)
	}
}

func newServer() * http.Server {
	mux := http.NewServeMux()
	mux.HandleFunc("/health", healthHandler)

	return &http.Server{
		Addr:    ":8080",
		Handler: mux,
	}
}

func main() {
	server := newServer()

	log.Println("API listening on :8080")

	if err := server.ListenAndServe(); err != nil {
		log.Fatal(err)
	}
}
