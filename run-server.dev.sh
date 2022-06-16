#!/usr/bin/env bash

set -e

cd server || exit

source .env

docker-compose down
docker-compose up -d --build --force-recreate kafka zookeeper nginx

./init-kafka.sh
