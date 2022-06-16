#!/usr/bin/env bash

set -e

echo "Environment prepared."

echo "Copy local dependencies into components."
cp -r ./pong ./client/wasm/
cp -r ./pong ./server/

echo "Start docker containers."
docker-compose down
docker-compose up -d --build --force-recreate kafka zookeeper nginx

echo "Initialize kafka."
./init-kafka.sh
