package main

import (
	"log"
	"net"
	"os/user"

	"github.com/alvidir/tp-auth/model/app"
	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/mysql"
	srv "github.com/alvidir/tp-auth/service/session"
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

func test() {
	db, err := mysql.OpenStream()
	if err != nil {
		log.Fatalf("Got %v error while opening stream", err.Error())
		return
	}

	// Migració de structs del Model (Es fa automatica si tenen els tags ben definits).
	db.AutoMigrate(&app.App{}, &user.User{}, &client.Client{})

	// Afegir files a les taules de la BBDD. Em suposo que se li pot passar l'struct del model ja construit, no cal construir-lo "in situ".
	app := app.New("tp-auth", "localhost:9090", "1234")
	db.Create(app)
}

func main() {
	// to change the flags on the default logger
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	if err := godotenv.Load(); err != nil {
		log.Panicf(errDotenvConfig, err.Error())
	}

	test()
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
