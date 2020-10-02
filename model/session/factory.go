package session

import (
	"github.com/alvidir/tp-auth/model/client"
)

// NewSession builds a brand new client with a provided password
func NewSession(client client.Client) Session {
	return &session{}
}
