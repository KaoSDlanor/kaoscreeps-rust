FROM rust:1.62.1
WORKDIR /usr/src/kaoscreeps-rust

RUN cargo install --git https://github.com/rustyscreeps/cargo-screeps --branch bindgen cargo-screeps

COPY . .

RUN cargo screeps upload