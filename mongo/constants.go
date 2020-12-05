package mongo

import "time"

const (
	// EnvMongoUsr represents the environment variable where the mongo's user key is located
	EnvMongoUsr = "MONGO_USR"
	// EnvMongoPwd represents the environment variable where the mongo's password key is located
	EnvMongoPwd = "MONGO_PWD"

	// Database consultable by the app
	Database = "tp-auth"
	// Timeout for any database request
	Timeout = 3600 * time.Second

	errNoMongoURI  = "No mongo uri has been provided"
	errNoMongoUsr  = "No mongo user has been provided"
	errNoMongoPwd  = "No mongo password has been provided"
	errNoMongoAuth = "No mongo auth has been provided"

	mongoURI = "mongodb+srv://%s:%s@cluster0.itrrv.mongodb.net/%s?retryWrites=true&w=majority"
)
