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

		// Регистрируем клиента
		if _, ok := clients[key]; !ok {
			log.Printf("[relay] New client: %s", key)
		}
		clients[key] = &Client{addr: remoteAddr, last: time.Now()}

		// Пересылаем пакет всем остальным клиентам
		data := make([]byte, n)
		copy(data, buf[:n])

		for k, c := range clients {
			if k == key {
				continue
			}
			// Удаляем старых клиентов (>30 сек)
			if time.Since(c.last) > 30*time.Second {
				delete(clients, k)
				continue
			}
			_, err := conn.WriteToUDP(data, c.addr)
			if err != nil {
				log.Printf("[relay] Write error to %s: %v", k, err)
			}
		}
	}
}