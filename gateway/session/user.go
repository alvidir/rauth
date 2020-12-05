package session

import (
	"context"

	"github.com/alvidir/tp-auth/model/session"
)

type sessionGateway struct {
	session.Controller
	ctx context.Context
}

func (gw *sessionGateway) Insert() error {
	return nil
}

func (gw *sessionGateway) Update() error {
	return nil
}

func (gw *sessionGateway) Remove() error {
	return nil
}
