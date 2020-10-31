package mysql

import "time"

const (
	// EnvMysqlUsr represents the environment variable where the mysql's user key is located
	EnvMysqlUsr = "MYSQL_USR"
	// EnvMysqlPwd represents the environment variable where the mysql's password key is located
	EnvMysqlPwd = "MYSQL_PWD"
	// EnvMysqlDB represents the environment variable where the mysql's database name is located
	EnvMysqlDB = "MYSQL_DB"
	// EnvMysqlURI represents the environment variable where the mysql's uri string is located
	EnvMysqlURI = "MYSQL_URI"

	errEnvNotFound = "Environment variable %s cannot be found"

	mysqlTimeout = 10 * time.Second
	mysqlDriver  = "mysql"
)
