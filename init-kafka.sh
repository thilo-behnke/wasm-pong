#!/usr/bin/env bash

set -e

source .env

docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic session --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic host_tick --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic peer_tick --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
docker exec pong_server_kafka /opt/bitnami/kafka/bin/kafka-topics.sh --create --topic heart_beat --bootstrap-server "$KAFKA_HOST:$KAFKA_PORT"
