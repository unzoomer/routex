package auth

import (
	"database/sql"
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	_ "github.com/mattn/go-sqlite3"
	"golang.org/x/crypto/bcrypt"
)

var (
	db        *sql.DB
	jwtSecret = []byte("routex-secret-change-in-production")
)

type User struct {
	ID           int64
	Email        string
	PasswordHash string
	SubActive    bool
	SubExpiry    time.Time
	CreatedAt    time.Time
}

type Server struct {
	port string
}

func New(port string) *Server {
	return &Server{port: port}
}

func (s *Server) Start() error {
	var err error
	db, err = sql.Open("sqlite3", "/var/lib/routex/users.db")
	if err != nil {
		return err
	}

	if err := migrate(); err != nil {
		return err
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/auth/register", handleRegister)
	mux.HandleFunc("/auth/login", handleLogin)
	mux.HandleFunc("/auth/verify", handleVerify)
	mux.HandleFunc("/auth/health", handleHealth)

	log.Printf("[auth] Listening on :%s", s.port)
	return http.ListenAndServe(":"+s.port, mux)
}

func migrate() error {
	_, err := db.Exec(`
		CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			email TEXT UNIQUE NOT NULL,
			password_hash TEXT NOT NULL,
			sub_active INTEGER DEFAULT 0,
			sub_expiry DATETIME,
			created_at DATETIME DEFAULT CURRENT_TIMESTAMP
		)
	`)
	return err
}

// POST /auth/register
func handleRegister(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", 405)
		return
	}

	var req struct {
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		jsonError(w, "invalid json", 400)
		return
	}

	if !strings.Contains(req.Email, "@") || len(req.Password) < 6 {
		jsonError(w, "invalid email or password too short", 400)
		return
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(req.Password), 12)
	if err != nil {
		jsonError(w, "server error", 500)
		return
	}

	_, err = db.Exec(
		"INSERT INTO users (email, password_hash) VALUES (?, ?)",
		strings.ToLower(req.Email), string(hash),
	)
	if err != nil {
		if strings.Contains(err.Error(), "UNIQUE") {
			jsonError(w, "email already registered", 409)
			return
		}
		jsonError(w, "server error", 500)
		return
	}

	log.Printf("[auth] New user: %s", req.Email)
	jsonOK(w, map[string]string{"message": "registered successfully"})
}

// POST /auth/login
func handleLogin(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", 405)
		return
	}

	var req struct {
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		jsonError(w, "invalid json", 400)
		return
	}

	var user User
	var subExpiry sql.NullTime
	err := db.QueryRow(
		"SELECT id, email, password_hash, sub_active, sub_expiry FROM users WHERE email = ?",
		strings.ToLower(req.Email),
	).Scan(&user.ID, &user.Email, &user.PasswordHash, &user.SubActive, &subExpiry)

	if errors.Is(err, sql.ErrNoRows) {
		jsonError(w, "invalid credentials", 401)
		return
	}
	if err != nil {
		jsonError(w, "server error", 500)
		return
	}

	if err := bcrypt.CompareHashAndPassword(
		[]byte(user.PasswordHash), []byte(req.Password),
	); err != nil {
		jsonError(w, "invalid credentials", 401)
		return
	}

	if subExpiry.Valid {
		user.SubExpiry = subExpiry.Time
	}

	// Проверяем подписку
	subActive := user.SubActive && time.Now().Before(user.SubExpiry)

	// Генерируем JWT
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"user_id":    user.ID,
		"email":      user.Email,
		"sub_active": subActive,
		"exp":        time.Now().Add(30 * 24 * time.Hour).Unix(),
	})

	tokenStr, err := token.SignedString(jwtSecret)
	if err != nil {
		jsonError(w, "server error", 500)
		return
	}

	log.Printf("[auth] Login: %s (sub=%v)", user.Email, subActive)
	jsonOK(w, map[string]interface{}{
		"token":      tokenStr,
		"email":      user.Email,
		"sub_active": subActive,
		"sub_expiry": user.SubExpiry,
	})
}

// GET /auth/verify — проверка токена
func handleVerify(w http.ResponseWriter, r *http.Request) {
	tokenStr := r.Header.Get("Authorization")
	tokenStr = strings.TrimPrefix(tokenStr, "Bearer ")

	token, err := jwt.Parse(tokenStr, func(t *jwt.Token) (interface{}, error) {
		return jwtSecret, nil
	})

	if err != nil || !token.Valid {
		jsonError(w, "invalid token", 401)
		return
	}

	claims := token.Claims.(jwt.MapClaims)
	jsonOK(w, map[string]interface{}{
		"valid":      true,
		"email":      claims["email"],
		"sub_active": claims["sub_active"],
	})
}

func handleHealth(w http.ResponseWriter, r *http.Request) {
	jsonOK(w, map[string]string{"status": "ok"})
}

func jsonOK(w http.ResponseWriter, v interface{}) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(v)
}

func jsonError(w http.ResponseWriter, msg string, code int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	json.NewEncoder(w).Encode(map[string]string{"error": msg})
}
