package main

import (
	"log"
	"net"

	"github.com/alvidir/tp-auth/model/app"
	"github.com/alvidir/tp-auth/mysql"
	srv "github.com/alvidir/tp-auth/service/session"
	"github.com/alvidir/util/config"
	"github.com/joho/godotenv"
	"google.golang.org/genproto/googleapis/cloud/location"
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

	defaultPort    = "9090"
	defaultNetwork = "tcp"
)

func getMainEnv() ([]string, error) {
	return config.CheckNemptyEnv(
		envPortKey, /*0*/
		envNetwKey /*1*/)
}

func test() {
	db, err := mysql.OpenStream()
	if err != nil {
		log.Printf("Got %v error while opening stream", err.Error())
		return
	}

	// Migració de structs del Model (Es fa automatica si tenen els tags ben definits).
	// db.AutoMigrate(&service.Service{})

	// Afegir files a les taules de la BBDD. Em suposo que se li pot passar l'struct del model ja construit, no cal construir-lo "in situ".
	db.Create(&app.App{
		Name:        "tp-auth",
		Description: "description of service test",
		Kind:        1,
		Location: location.Location{
			Name:        "location test",
			Address:     "address test",
			Coordinates: "101010",
			Extension:   10},
		Products: []product.Product{{
			Name:        "product test",
			Description: "description of product test",
			Price:       10,
			Status:      1}}})

}

func main() {
	// to change the flags on the default logger
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	if err := godotenv.Load(); err != nil {
		log.Panicf(errDotenvConfig, err.Error())
	}

	envs, err := getMainEnv()
	if err != nil {
		log.Fatalf(errConfigFailed, err.Error())
	}

	log.Printf(infoSetup, network, address)

	server := grpc.NewServer()
	service := srv.ImplementedSessionServer()
	service.RegisterServer(server)

	lis, err := net.Listen(network, address)
	if err != nil {
		log.Fatalf(errListenFailed, err)
	} else {

	}

	if err := server.Serve(lis); err != nil {
		log.Fatalf(errServeFailed, err)
	}

	log.Print(infoDone)
}
