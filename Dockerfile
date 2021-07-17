FROM rust:1.52 as builder
WORKDIR /usr/app
COPY . .

RUN cargo build --release
RUN strip ./target/release/somebotty

FROM debian:buster-slim as runtime
WORKDIR /usr/app
RUN groupadd -g 999 somebotty && \
    useradd -r -u 999 -g somebotty somebotty
COPY --chown=somebotty:somebotty --from=builder /usr/app/target/release/somebotty .

USER somebotty
EXPOSE 8080
ENTRYPOINT ["/usr/app/somebotty"]