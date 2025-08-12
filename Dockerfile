FROM node:22.18.0-slim

LABEL author="ilagan@amazon.com"
LABEL maintainer="ilagan@amazon.com"
LABEL description="Custom CI build image for an AWS Amplify-hosted React webapp + Rust-based WebAssembly."

# :: Rust will refers to these env vars.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# :: Update + install necessary tooling first (see above).
RUN apt update \
    && apt install -y curl git \
    && apt install -y --no-install-recommends ca-certificates gcc libc6-dev make

# :: Install rust via the rustup script.
#    This will install both the Rust compiler (rustc) and Cargo (cargo).
#    @see https://rustup.rs
RUN curl --proto '=https' --tlsv1.2 -sSf --output rustup https://sh.rustup.rs \
    && chmod +x ./rustup \
    && ./rustup -y --no-modify-path --default-toolchain nightly \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# :: Install wasm-pack via wasm-pack's init script.
#    @see https://rustwasm.github.io/wasm-pack/installer
RUN curl --proto '=https' --tlsv1.2 -sSf --output wasm-pack-init https://rustwasm.github.io/wasm-pack/installer/init.sh \
    && chmod +x ./wasm-pack-init \
    && ./wasm-pack-init

# :: Perform various cleanup tasks.
RUN rm ./rustup ./wasm-pack-init \
    && rm -rf /var/lib/apt/lists/*

RUN echo export PATH="$PATH" >> ~/.bashrc

ENTRYPOINT ["bash", "-c"]
