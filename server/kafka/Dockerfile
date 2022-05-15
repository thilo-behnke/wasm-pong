FROM bitnami/kafka:latest

RUN apt-get update -y
RUN apt-get install libc6

ADD custom-entrypoint.sh /
COPY kafka-script-proxy/target/release/kafka-script-proxy /bin/

ENTRYPOINT []
CMD [ "/custom-entrypoint.sh" ]
