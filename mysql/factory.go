package mysql

import (
	"database/sql"
	"fmt"

	// required as database driver connection
	_ "github.com/go-sql-driver/mysql"

	"github.com/alvidir/util/config"
)

func getMongoURI() (uri string, err error) {
	var url, username, password, database string
	if uri, err = config.CheckEnv(EnvMysqlURI); err != nil {
		return
	}

	if username, err = config.CheckEnv(EnvMysqlUsr); err != nil {
		return
	}

	if password, err = config.CheckEnv(EnvMysqlPwd); err != nil {
		return
	}

	if database, err = config.CheckEnv(EnvMysqlDB); err != nil {
		return
	}

	uri = fmt.Sprintf(url, username, password, database)
	return
}

// NewMysqlConnection returns a brand new connection to default database
func NewMysqlConnection() (db *sql.DB, err error) {
	var uri string
	if uri, err = getMysqlURI(); err != nil {
		return
	}

	return sql.Open(mysqlDriver, uri)
}
