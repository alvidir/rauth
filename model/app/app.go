package app

import (
	"github.com/alvidir/tp-auth/model/client"
	"gorm.io/gorm"
)

// An App represets a client of kind app
type App struct {
	gorm.Model
	ID          uint           `json:"id" gorm:"primaryKey; autoIncrement:true"`
	Name        string         `json:"name" gorm:"not null;unique"`
	Description string         `json:"description"`
	URL         string         `json:"url" gorm:"not null;unique"`
	Client      *client.Client `json:"-" gorm:"polymorphic:Owner;"`
}

// GetName return the app name
func (app *App) GetName() string {
	return app.Name
}

// GetDescription return the app name
func (app *App) GetDescription() string {
	return app.Description
}

// GetURL returns the app url
func (app *App) GetURL() string {
	return app.URL
}

// GetClient returns the user's client
func (app *App) GetClient() *client.Client {
	return app.Client
}
