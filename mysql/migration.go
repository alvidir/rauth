package mysql

import (
	"os/user"

	"github.com/alvidir/tp-auth/model/app"
	"github.com/alvidir/tp-auth/model/client"
	"xorm.io/xorm"
)

// MigrateTables migrates the database tables
func MigrateTables() (err error) {
	var engine *xorm.Engine
	if engine, err = OpenStream(); err != nil {
		return
	}

	defer engine.Close()
	return engine.Sync2(
		&client.Client{},
		&user.User{},
		&app.App{},
	)
}
