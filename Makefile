# Global about the project
VERSION=0.2.0
REPO=alvidir
PROJECT=oauth

install:
	sudo apt install libpq-dev
	sudo apt install pkg-config libssl-dev
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
	# add/remove -d in order to run containers at background or not

undeploy:
	podman-compose -f docker-compose.yaml down

reset: undeploy deploy migration

run:
	RUST_LOG=INFO cargo run

checks:
	curl -v localhost:5050

setup:
	python3 scripts/build_db_setup_script.py

test:
	cargo test -- --nocapture
	cargo test -- --nocapture --ignored

secrets:
	mkdir -p .ssh/
	openssl ecparam -name prime256v1 -genkey -noout -out .ssh/ec_key.pem
	openssl ec -in .ssh/ec_key.pem -pubout -out .ssh/ec_pubkey.pem
	openssl pkcs8 -topk8 -nocrypt -in .ssh/ec_key.pem -out .ssh/pkcs8_key.pem