FROM jayjohnson/rust-restapi-base:latest

RUN echo "cleaning previous build" \
    && rm -rf \
        /server/Cargo.toml \
        /server/Cargo.lock \
        /server/certs \
        /server/jwt \
        /server/src \
        /server/tests \
        /server/examples \
        /server/target/debug

ADD ./Cargo.toml /server/Cargo.toml
ADD ./Cargo.lock /server/Cargo.lock
ADD ./jwt /server/jwt
ADD ./src /server/src
ADD ./tests /server/tests
ADD ./tls /server/certs/tls
ADD ./examples /server/examples

RUN echo "starting build" \
    && cd /server \
    && cargo build --release --example server

EXPOSE 3000
ENTRYPOINT ["/server/target/release/examples/server"]
