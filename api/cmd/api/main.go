package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	qerv1 "github.com/noahlavelle/qer/gen/qer/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type healthResponse struct {
	Status string `json:"status"`
}

type engineClient struct {
	client qerv1.QueueEngineClient
}

type healthChecker interface {
	getCheckHealth(context.Context) (healthResponse, error)
}

func newEngineClient(conn *grpc.ClientConn) *engineClient {
	return &engineClient{
		client: qerv1.NewQueueEngineClient(conn),
	}
}

func (c *engineClient) getCheckHealth(ctx context.Context) (healthResponse, error) {
	var healthResponse healthResponse

	res, err := c.client.CheckHealth(ctx, &qerv1.CheckHealthRequest{})
	if err != nil {
		return healthResponse, fmt.Errorf("get engine health: %w", err)
	}

	healthResponse.Status = res.GetStatus()
	return healthResponse, nil
}

type server struct {
	engineClient healthChecker
}

func newServer(engineClient healthChecker) *server {
	return &server{
		engineClient: engineClient,
	}
}

func (s *server) healthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	response := healthResponse{
		Status: "ok",
	}

	if err := json.NewEncoder(w).Encode(response); err != nil {
		http.Error(w, "failed to encode response", http.StatusInternalServerError)
		return
	}
}

func (s *server) readyHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	ctx, cancel := context.WithTimeout(r.Context(), 2*time.Second)
	defer cancel()

	response, err := s.engineClient.getCheckHealth(ctx)
	if err != nil {
		http.Error(w, "engine unavailable", http.StatusInternalServerError)
		return
	}

	if err := json.NewEncoder(w).Encode(response); err != nil {
		http.Error(w, "failed to encode response", http.StatusInternalServerError)
		return
	}
}

func (s *server) routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/health", s.healthHandler)
	mux.HandleFunc("/ready", s.readyHandler)
	return mux
}

func main() {
	engineAddress := os.Getenv("ENGINE_ADDR")
	if engineAddress == "" {
		engineAddress = "engine:50051"
	}

	conn, err := grpc.NewClient(
		engineAddress,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	engineClient := newEngineClient(conn)
	srv := newServer(engineClient)

	httpServer := &http.Server{
		Addr:    ":8080",
		Handler: srv.routes(),
	}

	log.Println("API listening on :8080")

	if err := httpServer.ListenAndServe(); err != nil {
		log.Fatal(err)
	}
}
