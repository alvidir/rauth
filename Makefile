install:
	### fedora ###
	sudo dnf install postgresql-devel
	sudo dnf install pkg-config openssl-devel

	### debian ###
	# sudo apt install libpq-dev
	# sudo apt install pkg-config libssl-dev

build:
	podman build -t alvidir/rauth:latest -f ./container/rauth/containerfile .

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
	podman-compose  -f compose.yaml up --remove-orphans

follow:
	podman logs --follow --names rauth-server
	
undeploy:
	podman-compose -f compose.yaml down

run:
	RUST_LOG=INFO cargo run

test:
	RUST_BACKTRACE=full cargo test -- --nocapture