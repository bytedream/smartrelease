FROM rust:1.57-alpine

WORKDIR /smartrelease

COPY . .

RUN apk update && \
    apk add musl-dev && \
    rm -rf /var/cache/apk

RUN cargo build --release && \
    ln -s target/release/smartrelease .

EXPOSE 8080

ENTRYPOINT ["./smartrelease"]
