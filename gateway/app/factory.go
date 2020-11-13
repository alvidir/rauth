package app

import (
	"sync"

	"github.com/alvidir/tp-auth/model/app"
	"github.com/alvidir/tp-auth/mysql"
	"gorm.io/gorm"
)

var once sync.Once

// OpenAppStream opens an stream ensuring the apps's table does exists
func OpenAppStream() (db *gorm.DB, err error) {
	if db, err = mysql.OpenStream(); err != nil {
		return
	}

	once.Do(func() {
		// Automigrate must be called only once for each gateway, and allways on the stream's opening call.
		// This makes sure the client struct has its own table on the database. So model updates are only
		// migrable to the database rebooting the server (not on-the-run).
		db.AutoMigrate(&app.App{})
	})

	return
}
