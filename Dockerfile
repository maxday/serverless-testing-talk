FROM rust AS build

RUN cargo new /app/pizza
COPY Cargo.toml /app/pizza

WORKDIR /app/pizza
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY ./src /app/pizza/src

RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  touch /app/pizza/src/main.rs
  cargo test --bins
  cargo build --release
EOF

FROM ubuntu:jammy as compresser
RUN apt-get update
RUN apt-get install -y zip
RUN mkdir -p /bin
WORKDIR /bin
COPY --from=build /app/pizza/target/release/pizza /bin/bootstrap
RUN zip -r bootstrap.zip /bin/bootstrap

#keep the smallest possible docker image
FROM scratch
COPY --from=compresser /bin/bootstrap.zip /
ENTRYPOINT ["/bootstrap.zip"]