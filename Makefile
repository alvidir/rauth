# Global about the project
VERSION=0.1.0
REPO=alvidir
PROJECT=tp-auth

proto:
	protoc --go_out=plugins=grpc:. --go_opt=paths=source_relative proto/client/*.proto

build:
	docker build -t ${REPO}/${PROJECT}:${VERSION}-envoy -f ./docker/envoy/dockerfile .
	docker build -t ${REPO}/${PROJECT}:${VERSION} -f ./docker/tp-auth/dockerfile .

migration:
	diesel migration run

deploy:
	docker-compose --env-file ./.env -f docker-compose.yaml up --remove-orphans -d

undeploy:
	docker-compose -f docker-compose.yaml down

run:
	cargo run