# Base Dockerfile from leptos.dev:
# https://book.leptos.dev/deployment/ssr.html
# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine AS builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen

# Copy files to builder
WORKDIR /work
COPY . .

# Fetch JavaScript extensions
WORKDIR /work/web

RUN npm install

# Go back and build server
WORKDIR /work

RUN cargo build --release

FROM rustlang/rust:nightly-alpine AS runner

WORKDIR /app

COPY --from=builder /work/target/release/inferno /app/
COPY --from=builder /work/site /app/site

ENV RUST_LOG="info"
ENV INFERNO_SITE_ROOT=./site
EXPOSE 8080

CMD ["/app/inferno"]
