FROM rust:1-buster
RUN mkdir -p /spotifytops/src
WORKDIR /spotifytops/src
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY src ./src/
COPY templates /root/templates/
RUN cargo build --release

CMD ["cargo", "run", "--release"]