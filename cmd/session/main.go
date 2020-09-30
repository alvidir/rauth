package main

import (
	"fmt"
	"log"
	"net"
	"os"

	srv "github.com/alvidir/authentication/service"
	"google.golang.org/grpc"
)

var (
	address = getAddress()
	network = getNetwork()
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
