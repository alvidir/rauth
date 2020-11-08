package session

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
)

// New builds a brand new session for the provided client
func New(client *client.Client) *Session {
	return &Session{
		CreatedAt: time.Now(),
		Touch:     time.Now(),
		Client:    client,
	}
}
