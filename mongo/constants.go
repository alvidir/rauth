package mongo

import "time"

const (
	// EnvMongoUsr represents the environment variable where the mongo's user key is located
	EnvMongoUsr = "MONGO_USR"
	// EnvMongoPwd represents the environment variable where the mongo's password key is located
	EnvMongoPwd = "MONGO_PWD"
	// EnvMongoDB represents the environment variable where the mongo's database name is located
	EnvMongoDB = "MONGO_DB"
	// EnvMongoURI represents the environment variable where the mongo's uri string is located
	EnvMongoURI = "MONGO_URI"

	mongoTimeout = 10 * time.Second
)