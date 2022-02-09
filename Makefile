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

build:
	podman build -t ${REPO}/${PROJECT}:${VERSION}-nginx -f ./docker/nginx/dockerfile .
	# podman build -t ${REPO}/${PROJECT}:${VERSION}-server -f ./docker/rauth/dockerfile .
	
setup:
	mkdir -p .ssh/

	openssl ecparam -name prime256v1 -genkey -noout -out .ssh/ec_key.pem
	openssl ec -in .ssh/ec_key.pem -pubout -out .ssh/ec_pubkey.pem
	openssl pkcs8 -topk8 -nocrypt -in .ssh/ec_key.pem -out .ssh/pkcs8_key.pem
	
	cat .ssh/ec_key.pem | base64 | tr -d '\n' > .ssh/ec_key.base64
	cat .ssh/ec_pubkey.pem | base64 | tr -d '\n' > .ssh/ec_pubkey.base64
	cat .ssh/pkcs8_key.pem | base64 | tr -d '\n' > .ssh/pkcs8_key.base64
	
	python3 scripts/build_db_setup_script.py

deploy:
	podman-compose  -f docker-compose.yaml up --remove-orphans

logs:
	podman logs --follow --names rauth-server
	
undeploy:
	podman-compose -f docker-compose.yaml down

run:
	RUST_LOG=INFO cargo run

test:
	RUST_BACKTRACE=full cargo test -- --nocapture

integration-test:
	RUST_BACKTRACE=full cargo test --features integration-test -- --nocapture