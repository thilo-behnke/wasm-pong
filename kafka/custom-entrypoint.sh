#!/usr/bin/env bash

/bin/kafka-script-proxy &
/opt/bitnami/scripts/kafka/entrypoint.sh "/opt/bitnami/scripts/kafka/run.sh"
