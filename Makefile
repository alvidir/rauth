# Global about the project
VERSION=0.1.0
REPO=alvidir
PROJECT=tp-auth
# Mysql variables
MYSQL_CONTAINER_NAME=mysql

proto:
	protoc --go_out=plugins=grpc:. --go_opt=paths=source_relative proto/session/*.proto
	protoc --go_out=plugins=grpc:. --go_opt=paths=source_relative proto/app/*.proto

build:
	docker build -t ${REPO}/${PROJECT}:${VERSION}-envoy -f ./docker/envoy/dockerfile .
	docker build -t ${REPO}/${PROJECT}:${VERSION} -f ./docker/tp-auth/dockerfile .

mysql:
	docker logs ${MYSQL_CONTAINER_NAME} 2>&1 | grep GENERATED
	docker exec -it ${MYSQL_CONTAINER_NAME} mysql -uroot -p

run:
	go run ./cmd/tp-auth/main.go

test:
	go clean -testcache
	go test -v ./...

deploy:
	docker-compose --env-file ./.env -f docker-compose.yaml up --remove-orphans

undeploy:
	docker-compose -f docker-compose.yaml down