package main

import (
	"log"
	"os"

	"routex-server/internal/relay"
)

func main() {
	port := os.Getenv("ROUTEX_PORT")
	if port == "" {
		port = "7777"
	}

	log.Printf("[RouteX] Starting relay server on :%s", port)

	r := relay.New(port)
	if err := r.Start(); err != nil {
		log.Fatalf("[RouteX] Fatal: %v", err)
	}
}