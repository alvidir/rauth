package mysql

import (
	"database/sql"
	"database/sql/driver"
	"time"

	"github.com/alvidir/util/config"
	"github.com/go-sql-driver/mysql"
	gormSqlDriver "gorm.io/driver/mysql"
	"gorm.io/gorm"
)

func initMysqlConn() (interface{}, error) {
	return newMysqlDriver()
}

func getMysqlEnv() ([]string, error) {
	return config.CheckNemptyEnv(
		EnvMysqlUsr,  /*0*/
		EnvMysqlPwd,  /*1*/
		EnvMysqlHost, /*2*/
		EnvMysqlNetw, /*3*/
		EnvMysqlDB,   /*4*/
		EnvDefaultTimeout /*5*/)
}

func newMysqlDriver() (driver driver.Connector, err error) {
	var envs []string
	if envs, err = getMysqlEnv(); err != nil {
		return
	}

	conn := mysql.NewConfig()
	var timeout time.Duration
	if timeout, err = time.ParseDuration(envs[5]); err != nil {
		return
	}

	conn.Loc = time.Local
	conn.Timeout = timeout
	conn.ReadTimeout = timeout
	conn.WriteTimeout = timeout

	conn.Addr = envs[2]
	conn.DBName = envs[4]
	conn.User = envs[0]
	conn.Passwd = envs[1]
	conn.Net = envs[3]
	conn.ParseTime = true

	return mysql.NewConnector(conn)
}

// OpenStream returns a gateway to the mysql database
func OpenStream() (gormDB *gorm.DB, err error) {
	var conn driver.Connector
	if conn, err = getConnInstance(); err == nil {
		db := sql.OpenDB(conn)
		gormDB, err = gorm.Open(gormSqlDriver.New(gormSqlDriver.Config{
			Conn: db,
		}), &gorm.Config{})
	}

	return
}
