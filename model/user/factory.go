package user

import "github.com/alvidir/tp-auth/model/client"

// New builds a brand new user as client
func New(nickname, email, pwd string) *User {
	var usr User
	usr.Nickname = nickname
	usr.Emails = []string{email}
	usr.Nickname = nickname
	usr.client = client.New(&usr, pwd)
	return &usr
}
