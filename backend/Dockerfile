FROM clockworklabs/spacetime AS builder

WORKDIR /build
USER root
COPY . .
RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    spacetime build --module-path . && \
    chmod +x entrypoint.sh

FROM clockworklabs/spacetime AS runner

WORKDIR /app
COPY --from=builder /build/target/wasm32-unknown-unknown/release/snake_online_backend.wasm .
COPY --from=builder /build/entrypoint.sh .
COPY --from=builder /build/spacetimedb.service /etc/systemd/system/

ENTRYPOINT [ "/app/entrypoint.sh" ]
