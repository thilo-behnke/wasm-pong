#!/usr/bin/env bash

source .env

docker-compose down
docker-compose up -d --build --force-recreate
docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic session --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic move --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic status --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic input --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
