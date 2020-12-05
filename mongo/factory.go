package mongo

import (
	"context"
	"fmt"
	"os"

	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

func getMongoURI() string {
	username := os.Getenv(EnvMongoUsr)
	password := os.Getenv(EnvMongoPwd)

	return fmt.Sprintf(mongoURI, username, password, Database)
}

// NewMongoClient returns a brand new client
func NewMongoClient(ctx context.Context) (client *mongo.Client, err error) {
	mongoCtx, cancel := context.WithTimeout(ctx, Timeout)
	defer cancel()

	uri := getMongoURI()
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

	db = client.Database(Database)
	return
}
