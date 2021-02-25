# Global about the project
VERSION=0.1.0
REPO=alvidir
PROJECT=tp-auth

proto:
	protoc --go_out=plugins=grpc:. --go_opt=paths=source_relative proto/client/*.proto

build:
	podman build -t ${REPO}/${PROJECT}:${VERSION}-envoy -f ./docker/envoy/dockerfile .
	podman build -t ${REPO}/${PROJECT}:${VERSION} -f ./docker/tp-auth/dockerfile .

migration:
	diesel migration run

deploy:
	podman-compose -f docker-compose.yaml up --remove-orphans -d

undeploy:
	podman-compose -f docker-compose.yaml down

run:
	cargo run

test:
	RUST_BACKTRACE=1
	cargo test -- --nocapture
