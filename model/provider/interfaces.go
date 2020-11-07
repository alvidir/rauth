package provider

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/model/session"
)

// A Provider represents a cookie-kind session's provider
type Provider interface {
	SetAutogen(b bool)
	SetPrefix(string)
	GetName() string
	NewSession(string, *client.Client) (*session.Session, error)
	GetSession(string) (*session.Session, error)
	DestroySession(string) error
	Purge(time.Time) int
}
