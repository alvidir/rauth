package user

import (
	"github.com/alvidir/tp-auth/model/client"
	"gorm.io/gorm"
)

// User represents a client of type user
type User struct {
	gorm.Model
	ID       uint           `json:"id" gorm:"primaryKey; autoIncrement:true"`
	Nickname string         `json:"nickname" gorm:"not null;unique"`
	Emails   []string       `json:"emails" gorm:"not null"`
	Client   *client.Client `json:"-" gorm:"polymorphic:Owner;"`
}

// GetURI returns the default uri for the user
func (user *User) GetURI() string {
	return user.Emails[0]
}

// GetNickname returns the name of this client
func (user *User) GetNickname() string {
	return user.Nickname
}

// GetEmails returns the main email of this client
func (user *User) GetEmails() []string {
	return user.Emails
}

// GetClient returns the user's client
func (user *User) GetClient() *client.Client {
	return user.Client
}
