package user

import (
	"github.com/alvidir/tp-auth/model/client"
)

// user represents a client of type user
type user struct {
	ID       uint          `json:"id" gorm:"primaryKey; autoIncrement:true"`
	Nickname string        `json:"nickname" gorm:"not null; unique"`
	Emails   []string      `json:"emails" gorm:"not null"`
	Client   client.Client `json:"-" gorm:"foreignKey:ClientID, unique"`
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
