package client

import "github.com/alvidir/tp-auth/model/client"

// A Gateway represents the way between a model's object and the database
type Gateway interface {
	client.Controller
	Insert() error
	Update() error
	Remove() error
}
