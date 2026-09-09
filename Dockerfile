# syntax=docker/dockerfile:1

# Replaces the old `server/deploy` bash script, which built on the host and kept the server alive
# in a tmux session. Everything it did now happens here, except `git pull` — the build context is
# whatever is checked out, so pulling is the deployer's job, not the image's.

# ---------------------------------------------------------------------------
# Build the wasm game and the rustdoc output
# ---------------------------------------------------------------------------
# Pinned rather than `rust:slim`, so a rebuild months from now produces the same binary.
# `rust-toolchain.toml` is deliberately not copied in: it asks for "stable", which would make
# rustup fetch a second toolchain over the one this image already has.
FROM rust:1.98-slim-bookworm AS build

# `cargo doc` builds for the host, and the host-only dependencies pull in reqwest -> openssl-sys,
# which needs the OpenSSL headers. The wasm target does not, but the docs do.
RUN apt-get update \
    && apt-get install --no-install-recommends -y pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# `.cargo/config.toml` carries the wasm linker flag, without which the wasm build fails
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
# `include_bytes!` pulls the piece images, sounds, font and opening book straight into the binary
COPY assets ./assets

RUN cargo build --target wasm32-unknown-unknown --release
RUN cargo doc --no-deps --release

# ---------------------------------------------------------------------------
# Serve it
# ---------------------------------------------------------------------------
FROM node:22-slim AS runtime

WORKDIR /app/server

RUN corepack enable

# Installed before NODE_ENV is set, because ts-node and typescript are devDependencies and the
# server runs the TypeScript directly rather than compiling it ahead of time
COPY server/package.json server/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

ENV NODE_ENV=production

COPY server/server.ts server/tsconfig.json server/index.html ./
COPY server/static ./static

# `server.ts` resolves the game and the docs as `../target/...` relative to its own directory, so
# the build artifacts have to land in the same layout they have in the repo
COPY --from=build /app/target/wasm32-unknown-unknown/release/chess-ai.wasm \
                  /app/target/wasm32-unknown-unknown/release/chess-ai.wasm
COPY --from=build /app/target/doc /app/target/doc

USER node

EXPOSE 3252

# The same command as the `serve` script in package.json. Invoking node directly keeps pnpm and
# corepack off the runtime path, where a non-root user cannot write to their caches.
CMD ["node", "--loader", "ts-node/esm", "server.ts"]
