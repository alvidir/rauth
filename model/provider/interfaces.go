package provider

import (
	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/model/session"
)

// A Provider represents a cookie-kind session's provider
type Provider interface {
	NewSession(string, client.Client) (session.Session, error)
	GetSession(string) (session.Session, error)
	DestroySession(string) error
	Purge(int64) int
}
