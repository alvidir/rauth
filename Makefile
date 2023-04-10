BINARY_NAME=rauth
VERSION?=latest
PKG_MANAGER?=dnf

all: binaries

binaries: install-deps
	cargo build --release

images:
	-podman build -t alvidir/$(BINARY_NAME):$(VERSION)-grpc -f ./container/grpc/containerfile .

push-images:
	@podman push alvidir/$(BINARY_NAME):$(VERSION)-grpc

install-deps:
	-$(PKG_MANAGER) install -y protobuf-compiler
	-$(PKG_MANAGER) install -y postgresql-devel
	-$(PKG_MANAGER) install -y openssl-devel
	-$(PKG_MANAGER) install -y pkg-config

clean:
	@-cargo clean
	@-rm -rf bin/                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     o
	@-rm -rf secrets/

clean-images:
	@-podman image rm alvidir/$(BINARY_NAME):$(VERSION)-grpc
	
test:
	@RUST_BACKTRACE=full cargo test -- --nocapture

secrets:
	@mkdir -p secrets/
	@openssl ecparam -name prime256v1 -genkey -noout -out secrets/ec_key.pem
	@openssl ec -in secrets/ec_key.pem -pubout -out secrets/ec_pubkey.pem
	@openssl pkcs8 -topk8 -nocrypt -in secrets/ec_key.pem -out secrets/pkcs8_key.pem
	@cat secrets/ec_key.pem | base64 | tr -d '\n' > secrets/ec_key.base64
	@cat secrets/ec_pubkey.pem | base64 | tr -d '\n' > secrets/ec_pubkey.base64
	@cat secrets/pkcs8_key.pem | base64 | tr -d '\n' > secrets/pkcs8_key.base64

deploy:
	@python3 scripts/build_db_setup_script.py
	@podman-compose -f compose.yaml up -d

undeploy:
	@podman-compose -f compose.yaml down