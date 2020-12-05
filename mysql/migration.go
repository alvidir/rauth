package mysql

import (
	"os/user"

	"github.com/alvidir/tp-auth/model/app"
	"github.com/alvidir/tp-auth/model/client"
)

// MigrateTables migrates the database tables
func MigrateTables() (err error) {
	db, err := OpenStream()
	if err != nil {
		return
	}

	db.AutoMigrate(
		&app.App{},
		&user.User{},
		&client.Client{},
	)

	return
}
