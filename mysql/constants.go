package mysql

import "time"

const (
	// EnvMysqlUsr represents the environment variable where the mysql's username is located
	EnvMysqlUsr = "MYSQL_USR"
	// EnvMysqlPwd represents the environment variable where the mysql's password is located
	EnvMysqlPwd = "MYSQL_PWD"
	// EnvMysqlDB represents the environment variable where the mysql's database name is located
	EnvMysqlDB = "MYSQL_DB"
	// EnvMysqlHost represents the environment variable where the mysql's host is located
	EnvMysqlHost = "MYSQL_HOST"
	// EnvMysqlPort represents the environment variable where the mysql's port is located
	EnvMysqlPort = "MYSQL_PORT"

	mysqlTimeout = 10 * time.Second
	mysqlDriver  = "mysql"
	mysqlURI     = "%s:%s@tcp(%s:%s)/%s"
)
