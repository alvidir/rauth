package client

import (
	"sync"

	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/mysql"
	"gorm.io/gorm"
)

var once sync.Once

// OpenClientStream opens an stream ensuring the client's table does exists
func OpenClientStream() (db *gorm.DB, err error) {
	if db, err = mysql.OpenStream(); err != nil {
		return
	}

	once.Do(func() {
		// Automigrate must be called only once for each gateway, and allways on the stream's opening call.
		// This makes sure the client struct has its own table on the database. So model updates are only
		// migrable to the database rebooting the server (not on-the-run).
		db.AutoMigrate(&client.Client{})
	})

	return
}
