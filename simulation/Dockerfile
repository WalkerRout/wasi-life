# todo; BIN_TARGET ARG when ENTRYPOINT decides to work properly

FROM --platform=$BUILDPLATFORM rust:1.80-slim-bullseye AS buildbase
RUN rustup target add wasm32-wasi
WORKDIR /app

FROM --platform=$BUILDPLATFORM buildbase AS buildserver
COPY src/ src/
COPY Cargo.toml Cargo.toml
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" \
    cargo build --bin wasmedge_life --target wasm32-wasi --release

FROM scratch AS server
COPY --link --from=buildserver /app/target/wasm32-wasi/release/wasmedge_life.wasm wasmedge_life.wasm
# cannot pass arg into the ENTRYPOINT instruction, 
# env also does not work here,
# neither does shell format
ENTRYPOINT [ "wasmedge_life.wasm" ]
