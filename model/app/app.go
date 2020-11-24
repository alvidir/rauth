package app

import (
	"github.com/alvidir/tp-auth/model/client"
)

// An App represets a client of kind app
type App struct {
	client.Client `json:"-" `
	Name          string `json:"name"`
	Description   string `json:"description"`
	URL           string `json:"url"`
}

// GetName return the app name
func (app *App) GetName() string {
	return app.Name
}

// GetDescription return the app name
func (app *App) GetDescription() string {
	return app.Description
}

// GetURI returns the app url
func (app *App) GetURI() string {
	return app.URL
}
