package app

import "github.com/alvidir/tp-auth/model/client"

// New builds a brand new app as client
func New(name, url, pwd string) *App {
	var app = &App{
		Name: name,
		URL:  url,
	}

	app.Client = client.New(app, pwd)
	return app
}
