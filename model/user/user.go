package user

import (
	"fmt"

	"github.com/alvidir/tp-auth/model/client"
)

// A User represents a client of type user
type User struct {
	client.Controller `xorm:"extends"` //`json:"-" gorm:"foreignKey:ClientID, unique"`
	ID                int64            `xorm:"pk autoincr"`     //`json:"id" gorm:"primaryKey; autoIncrement:true"`
	Default           string           `xorm:"not null unique"` //`json:"default" gorm:"not null, unique"`
	Emails            []string         //`json:"emails" gorm:"not null"`
}

// AddEmail returns the main email of this client
func (user *User) AddEmail(new string) (err error) {
	for _, email := range user.Emails {
		if email == new {
			return fmt.Errorf(errEmailAlreadyExists, new)
		}
	}

	user.Emails = append(user.Emails, new)
	return
}

// GetAddr returns the main email of this client
func (user *User) GetAddr() string {
	return user.Default
}
