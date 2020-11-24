package session

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
)

// New builds a brand new session for the provided client
func New(client client.Controller, cookie string) Controller {
	return &session{
		Controller:  client,
		CreatedAt:   time.Now(),
		TouchAt:     time.Now(),
		CookieValue: cookie,
	}
}
