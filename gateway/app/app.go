package app

import (
	"context"

	"github.com/alvidir/tp-auth/model/app"
)

type appGateway struct {
	app.Controller
	ctx context.Context
}

func (gw *appGateway) Insert() error {
	return nil
}

func (gw *appGateway) Update() error {
	return nil
}

func (gw *appGateway) Remove() error {
	return nil
}
