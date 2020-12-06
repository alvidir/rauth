package user

import "github.com/alvidir/tp-auth/model/client"

// A Controller represents an user client
type Controller interface {
	client.Controller
	GetAddr() string
}
