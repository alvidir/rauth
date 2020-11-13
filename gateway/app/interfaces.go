package app

import "github.com/alvidir/tp-auth/model/app"

// A Gateway represents the way between a model's object and the database
type Gateway interface {
	app.Controller
	Insert() error
	Update() error
	Remove() error
}
