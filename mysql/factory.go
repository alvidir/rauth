package mysql

import (
	"database/sql"
	"fmt"

	"github.com/alvidir/util/config"

	// required as database driver connection
	_ "github.com/go-sql-driver/mysql"
)

func getMysqlURI() (uri string, err error) {
	var envs []string
	if envs, err = config.CheckNemptyEnv(
		EnvMysqlUsr,
		EnvMysqlPwd,
		EnvMysqlHost,
		EnvMysqlPort,
		EnvMysqlDB); err != nil {
		return
	}

	uri = fmt.Sprintf(mysqlURI, envs[0], envs[1], envs[2], envs[3], envs[4])
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
