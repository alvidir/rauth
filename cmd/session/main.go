package main

import (
	"fmt"
	"log"
	"net"
	"os"

	srv "github.com/alvidir/tp-auth/service/session"
	"google.golang.org/grpc"
)

const (
	infoSetup = "The service is being started on %s%s"
	infoDone  = "The service has finished successfully"

	errListenFailed = "The service has failed listening: %v"
	errServeFailed  = "The service has failed serving: %v"

	envPortKey = "SERVICE_PORT"
	envNetwKey = "SERVICE_NETW"

	defaultPort    = "9090"
	defaultNetwork = "tcp"
)

func getNetwork() string {
	if value, ok := os.LookupEnv(envNetwKey); ok {
		return value
	}

	return defaultNetwork
}

func getAddress() (address string) {
	address = defaultPort
	if value, ok := os.LookupEnv(envPortKey); ok {
		address = value
	}

	if address[0] != ':' {
		address = fmt.Sprintf(":%s", address)
	}

	return
}

func main() {
	address := getAddress()
	network := getNetwork()
	log.Printf(infoSetup, network, address)

	server := grpc.NewServer()
	service := srv.ImplementedSessionServer()
	service.RegisterServer(server)

	lis, err := net.Listen(network, address)
	if err != nil {
		log.Panicf(errListenFailed, err)
	} else {

	}

	if err := server.Serve(lis); err != nil {
		log.Panicf(errServeFailed, err)
	}

	log.Print(infoDone)
}
