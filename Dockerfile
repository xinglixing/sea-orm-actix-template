FROM rust AS build
WORKDIR /usr/src
RUN apt-get update && apt-get upgrade -y && apt-get install -y build-essential git clang llvm-dev libclang-dev libssl-dev pkg-config libpq-dev brotli
RUN USER=root cargo new sea-orm-play
WORKDIR /usr/src/sea-orm-play
COPY Cargo.toml Cargo.lock ./
COPY entity ./entity
COPY migration ./migration
RUN cargo build --release
# Copy the source and build the application.
COPY src ./src
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_INCLUDE_DIR="/usr/include/openssl"
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=build /usr/local/cargo/bin/sea-orm-play .
# standard env
# COPY .env ./.env
COPY entity ./entity
COPY migration ./migration
RUN apt-get update && apt-get install -y libssl-dev pkg-config libpq-dev brotli
CMD ["/sea-orm-play"]