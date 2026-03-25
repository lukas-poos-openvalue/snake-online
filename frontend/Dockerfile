# Build stage 1: application
FROM node:25-alpine AS builder-app
COPY --from=ghcr.io/casey/just:1.48.0 /just /usr/local/bin

WORKDIR /build
COPY justfile .
COPY package*.json .
RUN just install
COPY . .
RUN just build

# Build stage 2: hosting
FROM rust:1.93-alpine AS builder-hosting
COPY --from=ghcr.io/casey/just:1.48.0 /just /usr/local/bin

WORKDIR /build
COPY justfile .
COPY hosting ./hosting
COPY --from=builder-app /build/dist ./dist
RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    just build-hosting

# Production stage
FROM gcr.io/distroless/base:nonroot-2c95b073a3cde90a79799d333bab3cea11ff6f8c
ARG port=8080

WORKDIR /app
COPY --from=builder-hosting /build/hosting/target/release/hosting .
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=${port}
EXPOSE ${port}
CMD ["./hosting"]
