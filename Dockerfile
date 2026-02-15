# Stage 1: Build the bsnext binary on Alpine (musl)
FROM rust:alpine AS builder

RUN apk add --no-cache build-base perl openssl-dev openssl-libs-static

WORKDIR /app
COPY . .
ENV CFLAGS="-std=gnu17"
ENV CI=true
RUN cargo build --release --bin bsnext

# Stage 2: Verify it runs on a clean Alpine
FROM alpine:latest

COPY --from=builder /app/target/release/bsnext /usr/local/bin/bsnext
RUN bsnext --help
CMD ["bsnext"]
