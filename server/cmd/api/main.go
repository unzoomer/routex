package main

import (
	"log"
	"routex-server/internal/api"
)

func main() {
	log.Println("[RouteX API] Starting on :8080")
	s := api.New("8080")
	if err := s.Start(); err != nil {
		log.Fatalf("[RouteX API] Fatal: %v", err)
	}
}
