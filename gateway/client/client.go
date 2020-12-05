package client

import (
	"context"

	"github.com/alvidir/tp-auth/model/client"
)

type clientGateway struct {
	client.Controller
	ctx context.Context
}

func (gw *clientGateway) Insert() error {
	return nil
}

func (gw *clientGateway) Update() error {
	return nil
}

func (gw *clientGateway) Remove() error {
	return nil
}
