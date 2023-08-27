FROM rust:alpine as builder

WORKDIR /smartrelease

COPY . .

RUN apk update && \
    apk add musl-dev && \
    rm -rf /var/cache/apk

RUN cargo build --release

FROM alpine:latest

COPY --from=builder /smartrelease/target/release/smartrelease .

EXPOSE 8080

ENTRYPOINT ["./smartrelease"]
