package session

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/model/namespace"
)

// A Session represents the single session for a client
type Session struct {
	ID          string               `json:"id" bson:"_id,omitempty"`
	CookieName  string               `json:"cookie_name" bson:"cookie_name"`
	CookieValue string               `json:"cookie" bson:"cookie_value"`
	CreatedAt   time.Time            `json:"created_at" bson:"created_at"`
	Deadline    time.Time            `json:"deadline" bson:"deadline"`
	Touch       time.Time            `json:"trouch" bson:"touch"`
	Namespace   *namespace.Namespace `json:"namespace" bson:"namespace"`
	client      *client.Client
}

// GetCookieName returns the cookie name of the session
func (session *Session) GetCookieName() string {
	return session.CookieName
}

// GetCookieValue returns the cookie value of the session
func (session *Session) GetCookieValue() string {
	return session.CookieValue
}
