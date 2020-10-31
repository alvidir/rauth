package mongo

import (
	"context"
	"fmt"
	"os"

	"github.com/alvidir/util/config"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

func getMongoURI() (uri string, err error) {
	var envs []string
	if envs, err = config.CheckNemptyEnv(EnvMongoURI,
		EnvMongoUsr,
		EnvMongoPwd,
		EnvMongoDB); err != nil {
		return
	}

	uri = fmt.Sprintf(envs[0], envs[1], envs[2], envs[3])
	return
}

// NewMongoClient returns a brand new client
func NewMongoClient(ctx context.Context) (client *mongo.Client, err error) {
	mongoCtx, cancel := context.WithTimeout(ctx, mongoTimeout)
	defer cancel()

	var uri string
	if uri, err = getMongoURI(); err != nil {
		return
	}

	options := options.Client().ApplyURI(uri)
	client, err = mongo.Connect(mongoCtx, options)

	if err != nil {
		return nil, err
	}

	return client, nil
}

// NewDatabaseConnection returns a brand new database connection
func NewDatabaseConnection(ctx context.Context) (db *mongo.Database, err error) {
	var client *mongo.Client
	if client, err = NewMongoClient(ctx); err != nil {
		return
	}

	database := os.Getenv(EnvMongoDB)
	db = client.Database(database)
	return
}
