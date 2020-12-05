package session

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
)

// A session represents the single session for a client
type session struct {
	client.Controller
	ID        string    `json:"id" bson:"_id,omitempty"`
	OldCookie string    `json:"-" bson:"old_cookie"`
	Cookie    string    `json:"cookie" bson:"cookie"`
	CreatedAt time.Time `json:"created_at" bson:"created_at"`
	Deadline  time.Time `json:"deadline" bson:"deadline"`
	TouchAt   time.Time `json:"touch" bson:"touch"`
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

func (session *session) MatchCookie(cookie string) bool {
	return session.Cookie == cookie
}
