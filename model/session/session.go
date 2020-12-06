package session

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
	pb "github.com/alvidir/tp-auth/proto/client"
)

// A session represents the single session for a client
type session struct {
	client.Controller
	Cookie    string    `json:"cookie" bson:"_id,omitempty"`
	CreatedAt time.Time `json:"created_at" bson:"created_at"`
	TouchAt   time.Time `json:"touch" bson:"touch"`
	Deadline  time.Time `json:"deadline" bson:"deadline"`
	Status    pb.Status `json:"status" bson:"status"`
}

func (session *session) GetCreatedAt() time.Time {
	return session.CreatedAt
}

func (session *session) GetTouchAt() time.Time {
	return session.TouchAt
}

func (session *session) GetDeadline() time.Time {
	return session.Deadline
}

func (session *session) MatchCookie(cookie string) bool {
	return session.Cookie == cookie
}

func (session *session) GetCookie() (cookie string) {
	return session.Cookie
}

func (session *session) SessionStatus() pb.Status {
	return session.Status
}
