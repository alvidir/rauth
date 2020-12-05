package app

import (
	"github.com/alvidir/tp-auth/model/client"
)

// An app represets a client of kind app
type app struct {
	ID          uint          `json:"id" gorm:"primaryKey; autoIncrement:true"`
	Name        string        `json:"name" gorm:"not null; unique"`
	Description string        `json:"description" gorm:"not null"`
	URL         string        `json:"url" gorm:"not null; unique"`
	Client      client.Client `json:"-" gorm:"foreignKey:ClientID; unique"`
}

// GetName return the app name
func (app *app) GetName() string {
	return app.Name
}

// GetDescription return the app name
func (app *app) GetDescription() string {
	return app.Description
}

// GetURI returns the app url
func (app *app) GetURI() string {
	return app.URL
}
