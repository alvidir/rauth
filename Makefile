# Global about the project
VERSION=1.0.0
REPO=alvidir
PROJECT=rauth

install:
	### ubuntu ###
	# sudo apt install libpq-dev
	# sudo apt install pkg-config libssl-dev
	
	### fedora ###
	sudo dnf install postgresql-devel
	sudo dnf install pkg-config openssl-devel

	### global ###
	cargo install diesel_cli --no-default-features --features postgres

all: build setup deploy

build:
	podman build -t ${REPO}/${PROJECT}:${VERSION}-envoy -f ./docker/envoy/dockerfile .
	podman build -t ${REPO}/${PROJECT}:${VERSION}-server -f ./docker/rauth/dockerfile .
	
setup:
	mkdir -p .ssh/
	
	### setting up required secrets ###
	openssl ecparam -name prime256v1 -genkey -noout -out .ssh/ec_key.pem
	openssl ec -in .ssh/ec_key.pem -pubout -out .ssh/ec_pubkey.pem
	openssl pkcs8 -topk8 -nocrypt -in .ssh/ec_key.pem -out .ssh/pkcs8_key.pem
	
	### base64 encoding for JWT's required public and private keys ###
	cat .ssh/ec_key.pem | base64 | tr -d '\n' > .ssh/ec_key.base64
	cat .ssh/ec_pubkey.pem | base64 | tr -d '\n' > .ssh/ec_pubkey.base64
	cat .ssh/pkcs8_key.pem | base64 | tr -d '\n' > .ssh/pkcs8_key.base64
	
	### setting up db scripts ###
	python3 scripts/build_db_setup_script.py

migration:
	diesel migration run

deploy:
	podman-compose  -f docker-compose.yaml up --remove-orphans

logs:
	podman logs --follow --names rauth-server
	
undeploy:
	podman-compose -f docker-compose.yaml down

run:
	RUST_LOG=INFO cargo run

tests:
	RUST_BACKTRACE=1 cargo test -- --nocapture