package app

// NewApp builds a brand new app as client
func NewApp(name, description, url string) *App {
	return &App{
		Name:        name,
		Description: description,
		URL:         url,
	}
}
