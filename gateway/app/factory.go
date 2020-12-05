package app

import (
	"context"

	"github.com/alvidir/tp-auth/model/app"
)

// New builds a gateway for the provided app
func New(ctx context.Context, app app.Controller) Gateway {
	return &appGateway{Controller: app, ctx: ctx}
}
