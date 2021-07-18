FROM rust:1.52 as builder
WORKDIR /usr/app
COPY . .

RUN cargo build --release
RUN strip ./target/release/somebotty

FROM debian:buster-slim as runtime
RUN apt-get update && \
    apt-get install libssl1.1 -y --no-install-recommends

WORKDIR /usr/app
RUN groupadd -g 999 somebotty && \
    useradd -r -u 999 -g somebotty somebotty && \
    mkdir -p somebotty-db \
    chown -R somebotty:somebotty somebotty-db
COPY --chown=somebotty:somebotty --from=builder /usr/app/target/release/somebotty .

USER somebotty
EXPOSE 8080
ENTRYPOINT ["/usr/app/somebotty"]