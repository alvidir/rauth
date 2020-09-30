package main

import (
	"context"
	"log"
	"os"

	pb "github.com/alvidir/tp-auth/proto/session"
	"google.golang.org/grpc"
)

const (
	infoSetup = "Testing connection to %s"
	infoDone  = "Testing has finished successfully"

	errNoEndpoint = "Got no value for the env variable %s"
	errConnection = "Connection has failed with error %s"
	errResponse   = "Got %s error from response"

	envServerAddr = "ENDPOINT"
)

func main() {
	serverAddr, ok := os.LookupEnv(envServerAddr)
	if !ok {
		log.Panicf(errNoEndpoint, envServerAddr)
	}

	log.Printf(infoSetup, serverAddr)
	conn, err := grpc.Dial(serverAddr, grpc.WithInsecure())
	if err != nil {
		log.Panicf(errConnection, err.Error())
	}

	defer conn.Close()
	client := pb.NewSessionClient(conn)
	request := &pb.LoginRequest{}

	ctx := context.Background()
	if _, err := client.Login(ctx, request); err != nil {
		log.Panicf(errResponse, err.Error())
	}

	log.Print(infoDone)
}
