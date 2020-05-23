FROM rust:1.43.1-stretch

RUN apt-get update
RUN apt-get install binutils-arm-linux-gnueabihf libudev-dev
RUN rustup target add armv7-unknown-linux-musleabihf

RUN mkdir -p /opt/src
RUN mkdir -p /opt/build
WORKDIR /opt/src

COPY . .
ENV SERVER_TARGET="armv7-unknown-linux-musleabihf"

RUN cargo build --release --workspace=server --bin=server --target ${SERVER_TARGET}
RUN cp -R ./target/${SERVER_TARGET}/release/* /opt/build
