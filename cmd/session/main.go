package main

import (
	"fmt"
	"log"
	"net"
	"os"

	"github.com/alvidir/session/service/session"
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

var (
	// slice of all services names that will be attended by the same grpc server
	services = []string{session.ServiceName}
)

func network() string {
	if value, ok := os.LookupEnv(envNetwKey); ok {
		return value
	}

	return defaultNetwork
}

func address() (address string) {
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
	address := address()
	network := network()
	log.Printf(infoSetup, network, address)

	server := grpc.NewServer()
	lis, err := net.Listen(network, address)
	if err != nil {
		log.Panicf(errListenFailed, err)
	} else {

	}

	if err := server.Serve(lis); err != nil {
		log.Panicf(errServeFailed, err)
	}

	// on finishing
	log.Print(infoDone)
}
