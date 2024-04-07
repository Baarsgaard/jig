FROM rust:alpine as builder

RUN apk add build-base

RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new --vcs none --color never --quiet /src/jig
WORKDIR /src/jig

COPY Cargo.toml Cargo.lock ./

RUN USER=root cargo build --target x86_64-unknown-linux-musl --release

COPY src ./src/
RUN touch src/main.rs
RUN USER=root cargo build --target x86_64-unknown-linux-musl --release


FROM alpine
RUN apk add git

WORKDIR /src
RUN git config --global --add safe.directory /src

COPY --from=builder /src/jig/target/x86_64-unknown-linux-musl/release/jig /
ENTRYPOINT ["/jig"]

# docker build -t jig-image .
# alias='docker run --rm -v "$HOME/.config/jig/config.toml:/root/.config/jig/config.toml" -v "${PWD}/:/src" -it jig-image'
