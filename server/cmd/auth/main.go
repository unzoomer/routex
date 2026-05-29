package main

import (
	"log"
	"os"
	"routex-server/internal/auth"
)

func main() {
	port := os.Getenv("AUTH_PORT")
	if port == "" {
		port = "8081"
	}
	log.Println("[RouteX Auth] Starting on :" + port)
	s := auth.New(port)
	if err := s.Start(); err != nil {
		log.Fatalf("Fatal: %v", err)
	}
}
