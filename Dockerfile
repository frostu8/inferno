# Base Dockerfile from leptos.dev:
# https://book.leptos.dev/deployment/ssr.html
# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine AS builder

RUN apk update && \
    apk add --no-cache openssl bash curl npm libc-dev binaryen

RUN npm install -g sass

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Copy files to builder
WORKDIR /work
COPY . .

# Build JavaScript extensions
WORKDIR /work/js

RUN npm install
RUN npx rollup -c --environment NODE_ENV:production

# Results shold be automagically placed in public/inferno.ext.js

# Go back and build server
WORKDIR /work

RUN cargo leptos build --release -vv

FROM rustlang/rust:nightly-alpine AS runner

WORKDIR /app

COPY --from=builder /work/target/release/inferno /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 8080

CMD ["/app/inferno"]
