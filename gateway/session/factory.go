package session

import (
	"context"

	"github.com/alvidir/tp-auth/model/session"
)

// New builds a gateway for the provided session
func New(ctx context.Context, session session.Controller) Gateway {
	return &sessionGateway{Controller: session, ctx: ctx}
}
