package user

import (
	"context"

	"github.com/alvidir/tp-auth/model/user"
)

// New builds a gateway for the provided user
func New(ctx context.Context, user user.Controller) Gateway {
	return &userGateway{Controller: user, ctx: ctx}
}
