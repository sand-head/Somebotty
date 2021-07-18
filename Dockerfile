FROM rust:1.52 as builder
WORKDIR /usr/app
COPY . .

RUN cargo build --release
RUN strip ./target/release/somebotty

FROM debian:buster-slim as runtime
RUN apt-get update && \
    apt-get install libssl1.1 -y --no-install-recommends

RUN groupadd -g 999 somebotty && \
    useradd -r -u 999 -g somebotty somebotty && \
    mkdir -p /usr/app/somebotty-db && \
    chown -R somebotty:somebotty /usr/app/somebotty-db
WORKDIR /usr/app
COPY --chown=somebotty:somebotty --from=builder /usr/app/target/release/somebotty .

USER somebotty:somebotty
EXPOSE 8080
ENTRYPOINT ["/usr/app/somebotty"]