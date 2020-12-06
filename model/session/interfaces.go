package session

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
	pb "github.com/alvidir/tp-auth/proto/client"
)

// A Controller represents the session of some client
type Controller interface {
	client.Controller
	GetCreatedAt() time.Time
	GetTouchAt() time.Time
	GetDeadline() time.Time
	MatchCookie(string) bool
	GetCookie() string
	SessionStatus() pb.Status
}
