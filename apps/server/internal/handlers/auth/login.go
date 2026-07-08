package auth

import (
	"html/template"
	"net/http"
)

var loginPage *template.Template

func init() {
	loginPage = template.Must(template.ParseFiles("templates/v1/auth/login/index.html"))
}

type AuthHandler struct{}

func NewAuthHandler() *AuthHandler {
	return &AuthHandler{}
}

func (h *AuthHandler) LoginPage(w http.ResponseWriter, r *http.Request) {
	err := loginPage.Execute(w, nil)
	if err != nil {
		http.Error(w, "failed to render login page", http.StatusInternalServerError)
		return
	}
}
