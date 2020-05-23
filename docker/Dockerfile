FROM rust:1.43.1-stretch

RUN apt-get update
RUN apt-get install libudev-dev

RUN mkdir -p /opt/src
RUN mkdir -p /opt/rc.machine
WORKDIR /opt/src

COPY . .

RUN cargo build --release --workspace=server --bin=server

RUN mv ./target/release/server /opt/rc.machine/
RUN mv /opt/src/Setting.toml /opt/rc.machine/

RUN rm -rf target
RUN rm -rf /opt/src

CMD ["/opt/rc.machine/server"]
