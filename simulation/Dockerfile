
FROM --platform=$BUILDPLATFORM rust:1.80-slim-bullseye AS buildbase
RUN rustup target add wasm32-wasi
WORKDIR /app

FROM --platform=$BUILDPLATFORM buildbase AS buildapp
ARG BIN_TARGET
COPY src/ src/
COPY Cargo.toml Cargo.toml
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" \
    cargo build --bin $BIN_TARGET --target wasm32-wasi --release
RUN chmod a+x ./target/wasm32-wasi/release/$BIN_TARGET.wasm

FROM scratch AS server
ARG BIN_TARGET
ENTRYPOINT [ "$BIN_TARGET.wasm" ]
COPY --link --from=buildapp /app/target/wasm32-wasi/release/$BIN_TARGET.wasm $BIN_TARGET.wasm
