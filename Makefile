# Global about the project
VERSION=0.2.0
REPO=alvidir
PROJECT=oauth

install:
	sudo apt install libpq-dev
	cargo install diesel_cli --no-default-features --features postgres

proto:
	protoc --go_out=plugins=grpc:. --go_opt=paths=source_relative proto/client/*.proto

build:
	podman build -t ${REPO}/${PROJECT}:${VERSION}-envoy -f ./docker/envoy/dockerfile .
	podman build -t ${REPO}/${PROJECT}:${VERSION}-server -f ./docker/oauth/dockerfile .

migration:
	diesel migration run

deploy:
	podman-compose -f docker-compose.yaml up --remove-orphans -d
	# delete -d in order to see output logs

undeploy:
	podman-compose -f docker-compose.yaml down

reset: undeploy deploy migration

run:
	cargo run

test:
	RUST_BACKTRACE=1
	cargo test -- --nocapture

check-envoy:
	curl -v localhost:5050

init-scripts:
	python3 scripts/build_db_init_script.py