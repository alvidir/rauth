package app

import (
	"github.com/alvidir/tp-auth/model/client"
)

// An App represets a client of kind app
type App struct {
	client.Controller `xorm:"extends"` //`json:"-" gorm:"foreignKey:ClientID; unique"`
	ID                int64            `xorm:"pk autoincr"`                            //`json:"id" gorm:"primaryKey; autoIncrement:true"`
	Description       string           `xorm:"varchar(255)"`                           //`json:"description" gorm:"not null"`
	URL               string           `xorm:"varchar(255) not null unique 'app_url'"` //`json:"url" gorm:"not null; unique"`
}

// GetDescription return the app name
func (app *App) GetDescription() string {
	return app.Description
}

// GetAddr returns the app url
func (app *App) GetAddr() string {
	return app.URL
}
