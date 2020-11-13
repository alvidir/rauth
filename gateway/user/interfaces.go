package user

import "github.com/alvidir/tp-auth/model/user"

// A Gateway represents the way between a model's object and the database
type Gateway interface {
	user.Controller
	Insert() error
	Update() error
	Remove() error
}
