package user

import (
	"context"
	"fmt"

	"github.com/alvidir/tp-auth/model/client"
)

// New builds a brand new client with a provided password
func New(client client.Controller, email string) Controller {
	user := &User{
		Controller: client,
		Default:    email,
		Emails:     []string{email},
	}

	client.SetExtension(user)
	return user
}

// FindUserByEmail returns the user with the given email
func FindUserByEmail(ctx context.Context, email string) (Controller, error) {
	return nil, fmt.Errorf("TODO")
}
