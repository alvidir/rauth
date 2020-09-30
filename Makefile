# Global about the project
VERSION=0.1.0
REPO=alvidir
PROJECT=tp-auth
# Volume variables
ROOT="${PWD}"
VOLUME_PATH=/tmp/${PROJECT}
# Mysql variables
MYSQL_CONTAINER_NAME=mysql
MYSQL_VOLUME_PATH="${VOLUME_PATH}/mysql"
# PHPMyAdmin variables
MYADMIN_CONTAINER_NAME=myadmin
# DATABASE network
DB_NETWORK_NAME=${PROJECT}.network.db

proto:
	protoc --go_out=plugins=grpc:. --go_opt=paths=source_relative proto/session/*.proto

build:
	docker build -t ${REPO}/${PROJECT}:${VERSION}-envoy -f ./docker/envoy/dockerfile .
	docker build -t ${REPO}/${PROJECT}:${VERSION} -f ./docker/session/dockerfile .

mysql:
	docker logs ${MYSQL_CONTAINER_NAME} 2>&1 | grep GENERATED
	docker exec -it ${MYSQL_CONTAINER_NAME} mysql -uroot -p