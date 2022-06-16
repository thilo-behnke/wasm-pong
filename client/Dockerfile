FROM rust:latest as rust_build_stage

RUN apt-get update && apt-get upgrade -y
RUN apt-get install curl

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

RUN mkdir -p /build/client/wasm
ADD ./wasm /build/client/wasm
ADD ./wasm/pong /build/pong
WORKDIR /build/client/wasm
RUN wasm-pack build

FROM node:12 as www_build_stage

RUN mkdir /build

ADD ./www /build/www
RUN ls /build/www
RUN mkdir /build/www/node_modules
WORKDIR /build/www

COPY --from=rust_build_stage /build/client/wasm/pkg/ /build/wasm/pkg/

RUN npm install
RUN npm run build

FROM nginx:latest

COPY --from=www_build_stage /build/www/dist/ /usr/share/nginx/pong/web

COPY ./nginx/default.conf /etc/nginx/conf.d/default.conf
COPY ./nginx/nginx.conf /etc/nginx/nginx.conf
