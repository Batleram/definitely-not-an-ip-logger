FROM rust:1.90-bullseye as notiplog-builder

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

COPY --from=notiplog-builder  /srv/target/release/notiplog-srv .
COPY --from=notiplog-builder  /srv/target/release/static/ ./static
COPY --from=notiplog-builder  /srv/target/release/templates/ ./templates
COPY --from=notiplog-builder  /srv/target/release/migrations/ ./migrations

EXPOSE $PORT

CMD ["./notiplog-srv"]

