#!/usr/bin/env bash

set -e

echo "Environment prepared."

echo "Hack for copying the main cargo.lock into the workspace members"
cp Cargo.toml ./pong/
cp Cargo.toml ./server/

echo "Copy local dependencies into components."
cp -r ./pong ./client/wasm/
cp -r ./pong ./server/

echo "Start docker containers."
docker-compose -f docker-compose.dev.yml down
docker-compose -f docker-compose.dev.yml up -d --build --force-recreate kafka zookeeper nginx

echo "Remove temporary local dependencies from components."
rm -rf ./client/wasm/pong
rm -rf ./server/pong

echo "Initialize kafka."
./init-kafka.sh
