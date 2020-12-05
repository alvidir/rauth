package session

import "github.com/alvidir/tp-auth/model/session"

// A Gateway represents the way between a model's object and the database
type Gateway interface {
	session.Controller
	Insert() error
	Update() error
	Remove() error
}
