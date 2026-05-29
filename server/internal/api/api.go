package api

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os/exec"
	"regexp"
	"sync"
)

type RegisterRequest struct {
	PublicKey string `json:"public_key"`
}

type RegisterResponse struct {
	Success  bool   `json:"success"`
	Message  string `json:"message"`
	ClientIP string `json:"client_ip"`
}

var (
	mu          sync.Mutex
	validKeyReg = regexp.MustCompile(`^[A-Za-z0-9+/]{43}=$`)
	nextIP      = 4 // 10.0.0.4, 10.0.0.5, ...
)

type Server struct {
	port string
}

func New(port string) *Server {
	return &Server{port: port}
}

func (s *Server) Start() error {
	http.HandleFunc("/api/register", handleRegister)
	http.HandleFunc("/api/health", handleHealth)
	log.Printf("[api] Listening on :%s", s.port)
	return http.ListenAndServe(":"+s.port, nil)
}

func handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
}

func handleRegister(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req RegisterRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}

	// Валидация ключа
	if !validKeyReg.MatchString(req.PublicKey) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(RegisterResponse{
			Success: false,
			Message: "invalid public key format",
		})
		return
	}

	mu.Lock()
	defer mu.Unlock()

	// Назначаем IP клиенту
	clientIP := fmt.Sprintf("10.0.0.%d", nextIP)
	nextIP++

	// Добавляем ключ в WireGuard
	cmd := exec.Command("wg", "set", "wg0",
		"peer", req.PublicKey,
		"allowed-ips", clientIP+"/32",
	)
	if out, err := cmd.CombinedOutput(); err != nil {
		log.Printf("[api] wg set error: %v, output: %s", err, out)
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(RegisterResponse{
			Success: false,
			Message: "failed to register key",
		})
		return
	}

	// Сохраняем конфиг
	exec.Command("wg-quick", "save", "wg0").Run()

	log.Printf("[api] Registered peer %s with IP %s", req.PublicKey[:8]+"...", clientIP)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(RegisterResponse{
		Success:  true,
		Message:  "registered successfully",
		ClientIP: clientIP,
	})
}
