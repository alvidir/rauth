package session

import (
	"sync"
	"time"

	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/model/namespace"
)

type npKey string
type npValue namespace.Controller

// A session represents the single session for a client
type session struct {
	client.Controller
	ID          string    `json:"id" bson:"_id,omitempty"`
	CookieName  string    `json:"cookie_name" bson:"cookie_name"`
	CookieValue string    `json:"cookie" bson:"cookie_value"`
	CreatedAt   time.Time `json:"created_at" bson:"created_at"`
	Deadline    time.Time `json:"deadline" bson:"deadline"`
	TouchAt     time.Time `json:"trouch" bson:"touch"`
	namespaces  sync.Map
}

func (session *session) GetNamespace(appName string) namespace.Controller {
	return nil
}

func (session *session) GetCreatedAt() time.Time {
	return session.CreatedAt
}

func (session *session) GetDeadline() time.Time {
	return session.Deadline
}

func (session *session) GetTouchAt() time.Time {
	return session.TouchAt
}

func (session *session) GetCookieName() string {
	return session.CookieName
}

func (session *session) MatchCookie(cookie string) bool {
	return session.CookieValue == cookie
}
