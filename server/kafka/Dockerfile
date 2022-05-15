FROM bitnami/kafka:latest

ADD custom-entrypoint.sh /
ADD kafka-script-proxy/target/release/kafka-script-proxy /bin/

ENTRYPOINT []
CMD [ "/custom-entrypoint.sh" ]
