FROM rust:1.54 as build-stage

RUN mkdir -p /opt/pong/kafka-script-proxy
WORKDIR /opt/pong/kafka-script-proxy
ADD kafka-script-proxy ./
RUN cargo build --release

FROM bitnami/kafka:latest

ADD custom-entrypoint.sh /
COPY --from=build-stage /opt/pong/kafka-script-proxy/target/release/kafka-script-proxy /bin/

USER root
RUN mkdir -p /var/log/kafka-script-proxy

ENTRYPOINT []
CMD [ "/custom-entrypoint.sh" ]
