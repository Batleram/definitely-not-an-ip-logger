FROM rust:1.75-buster as notiplog-builder

WORKDIR /srv

RUN echo "fn main() {}" > dummy.rs
COPY Cargo.toml Cargo.lock ./
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo fetch
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY . .
RUN cargo build --release

FROM debian:bookworm

WORKDIR /srv

COPY --from=notiplog-builder /srv/target/release .

EXPOSE $PORT

CMD ["./notiplog-srv"]

