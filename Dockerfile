FROM rust:1.75-buster as amg-builder

WORKDIR /srv

RUN echo "fn main() {}" > dummy.rs
COPY Cargo.toml .
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY . .
RUN cargo build --release

FROM debian:bookworm

WORKDIR /srv

COPY --from=amg-builder /srv/target/release .

EXPOSE $PORT

CMD ["./amg-srv"]

