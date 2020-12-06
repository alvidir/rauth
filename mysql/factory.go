package mysql

import (
	"database/sql/driver"
	"fmt"
	"log"
	"time"

	"github.com/alvidir/util/config"
	"github.com/go-sql-driver/mysql"
	"xorm.io/xorm"
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

	conn.User = envs[0]
	conn.Passwd = envs[1]
	conn.Addr = envs[2]
	conn.Net = envs[3]
	conn.DBName = envs[4]
	conn.ParseTime = true

	return mysql.NewConnector(conn)
}

func getMysqlURL() (url string, err error) {
	var envs []string
	if envs, err = getMysqlEnv(); err != nil {
		return
	}

	url = fmt.Sprintf(mysqlURI, envs[0], envs[1], envs[3], envs[2], envs[4])
	return
}

// OpenStream returns a gateway to the mysql database
func OpenStream() (engine *xorm.Engine, err error) {
	var url string
	if url, err = getMysqlURL(); err != nil {
		return
	}

	log.Println(url)
	return xorm.NewEngine("mysql", url)

	//var conn driver.Connector
	//if conn, err = getConnInstance(); err != nil {
	//	return
	//}
	//
	//db := sql.OpenDB(conn)
	//config := gormSqlDriver.Config{
	//	Conn: db,
	//}
	//
	//driver := gormSqlDriver.New(config)
	//return gorm.Open(driver, &gorm.Config{})
}
