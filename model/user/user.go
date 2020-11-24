package user

import (
	"github.com/alvidir/tp-auth/model/client"
)

// user represents a client of type user
type user struct {
	Client   client.Client `json:"-" bson:"client"`
	Nickname string        `json:"nickname" bson:"nickname"`
	Emails   []string      `json:"emails" bson:"emails"`
}

// GetURI returns the default uri for the user
func (user *user) GetURI() string {
	return user.Emails[0]
}

// GetNickname returns the name of this client
func (user *user) GetNickname() string {
	return user.Nickname
}

// GetMainEmail returns the main email of this client
func (user *user) GetMainEmail() string {
	return user.Emails[0]
}
