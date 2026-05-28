package relay

import (
	"fmt"
	"log"
	"net"
	"time"
)

type Server struct {
	port string
}

type Client struct {
	addr *net.UDPAddr
	last time.Time
}

func New(port string) *Server {
	return &Server{port: port}
}

func (s *Server) Start() error {
	addr, err := net.ResolveUDPAddr("udp", fmt.Sprintf(":%s", s.port))
	if err != nil {
		return err
	}

	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		return err
	}
	defer conn.Close()

	log.Printf("[relay] Listening on UDP :%s", s.port)

	clients := make(map[string]*Client)
	buf := make([]byte, 65536)

	for {
		n, remoteAddr, err := conn.ReadFromUDP(buf)
		if err != nil {
			log.Printf("[relay] Read error: %v", err)
			continue
		}

		key := remoteAddr.String()
		data := buf[:n]

		// Отвечаем на пинг
		if n == 4 && string(data) == "ping" {
			conn.WriteToUDP([]byte("pong"), remoteAddr)
			continue
		}

		if _, ok := clients[key]; !ok {
			log.Printf("[relay] New client: %s", key)
		}
		clients[key] = &Client{addr: remoteAddr, last: time.Now()}

		for k, c := range clients {
			if k == key {
				continue
			}
			if time.Since(c.last) > 30*time.Second {
				delete(clients, k)
				continue
			}
			outData := make([]byte, n)
			copy(outData, data)
			conn.WriteToUDP(outData, c.addr)
		}
	}
}
