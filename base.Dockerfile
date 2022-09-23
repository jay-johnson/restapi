FROM docker.io/rust:1.64.0

RUN apt-get update \
    && echo "installing openssl dependencies for tls encryption" \
    && apt-get install -y pkg-config \
    && apt-get install -y zlib1g cmake libssl-dev libsasl2-dev libzstd-dev \
    && apt-get install -y openssl \
    && echo "installing debugging tools" \
    && apt-get install -y \
        curl \
        procps \
        net-tools \
    && update-ca-certificates \
    && mkdir -p -m 777 /server

ADD ./Cargo.toml /server/Cargo.toml
ADD ./Cargo.lock /server/Cargo.lock
ADD ./certs /server/certs
ADD ./jwt /server/jwt
ADD ./src /server/src
ADD ./tests /server/tests
ADD ./examples /server/examples

# add custom user here in future version

RUN echo "starting build" \
    && cd /server \
    && cargo build --release --example server

EXPOSE 3000
ENTRYPOINT ["/server/target/release/examples/server"]
