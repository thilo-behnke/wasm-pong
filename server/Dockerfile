FROM rust:latest

RUN mkdir /build
ADD ./ /build
WORKDIR /build
RUN cargo build --release
