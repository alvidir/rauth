package user

import (
	"github.com/alvidir/tp-auth/model/client"
)

// A User represents an user client
type User interface {
	client.Extension
}
