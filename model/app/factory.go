package app

// New builds a brand new app as client
func New(name, description, url, pwd string) *App {
	var app App
	app.Name = name
	app.Description = description
	app.URL = url
	//app.Client = client.New(&app, pwd)
	return &app
}
