package user

import (
	"context"

	"github.com/alvidir/tp-auth/model/user"
)

type userGateway struct {
	user.Controller
	ctx context.Context
}

func (gw *userGateway) Insert() error {
	return nil
}

func (gw *userGateway) Update() error {
	return nil
}

func (gw *userGateway) Remove() error {
	return nil
}
