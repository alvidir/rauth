package main

import (
	"log"
	"net"

	"github.com/alvidir/tp-auth/mysql"
	srv "github.com/alvidir/tp-auth/service/client"
	"github.com/alvidir/util/config"
	"github.com/joho/godotenv"
	"google.golang.org/grpc"
)

const (
	infoSetup = "The service is being started on %s%s"
	infoDone  = "The service has finished successfully"

	errConfigFailed = "Got %s, while setting up service configuration"
	errDotenvConfig = "The service has failed setting up dotenv: %s"
	errListenFailed = "The service has failed listening: %s"
	errServeFailed  = "The service has failed serving: %s"

	envPortKey = "SERVICE_PORT"
	envNetwKey = "SERVICE_NETW"
)

func getMainEnv() ([]string, error) {
	return config.CheckNemptyEnv(
		envPortKey, /*0*/
		envNetwKey /*1*/)
}

func setup() (err error) {
	// to change the flags on the default logger
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	if err = godotenv.Load(); err != nil {
		return
	}

	if err = mysql.MigrateTables(); err != nil {
		return
	}

	//if err = clientTX.SetupDummyUser(); err != nil {
	//	return
	//}

	return
}

func main() {
	if err := setup(); err != nil {
		log.Fatalf(err.Error())
	}

	envs, err := getMainEnv()
	if err != nil {
		log.Fatalf(errConfigFailed, err.Error())
	}

	address := ":" + envs[0]
	log.Printf(infoSetup, envs[1], address)

	server := grpc.NewServer()
	service := srv.ImplementedSessionServer()
	service.RegisterServer(server)

	lis, err := net.Listen(envs[1], address)
	if err != nil {
		log.Fatalf(errListenFailed, err)
	}

	if err := server.Serve(lis); err != nil {
		log.Fatalf(errServeFailed, err)
	}

	log.Print(infoDone)
}
