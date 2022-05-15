FROM rust:latest as build-stage

RUN mkdir -p /opt/pong/
WORKDIR /opt/pong
ADD kafka-script-proxy ./
RUN cargo build --release

FROM bitnami/kafka:latest

ADD custom-entrypoint.sh /
COPY --from=build-stage /opt/pong/kafka-script-proxy/target/release/kafka-script-proxy /bin/

ENTRYPOINT []
CMD [ "/custom-entrypoint.sh" ]
