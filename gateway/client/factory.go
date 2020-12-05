package client

import (
	"context"

	"github.com/alvidir/tp-auth/model/client"
)

// New builds a gateway for the provided client
func New(ctx context.Context, client client.Controller) Gateway {
	return &clientGateway{Controller: client, ctx: ctx}
}
