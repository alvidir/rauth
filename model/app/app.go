package app

type app struct {
	name string
}

func (app *app) GetName() string {
	return app.name
}
