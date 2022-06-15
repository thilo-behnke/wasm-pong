FROM rust:latest as build_stage

RUN mkdir /build
ADD ./pong /build/pong
ADD ./server /build/server
WORKDIR /build/server
RUN cargo build --release

FROM ubuntu:20.04

RUN apt-get update && apt-get install libssl1.1

COPY --from=build_stage /build/server/target/release/server /usr/local/bin/
RUN chmod +x /usr/local/bin/server

CMD /usr/local/bin/server
