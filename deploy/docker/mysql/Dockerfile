# syntax=docker/dockerfile:experimental
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/app/target \
	cargo build --release && \
    # Copy executable out of the cache so it is available in the final image.
    cp target/release/server_api_all ./server_api_all && \
    cp target/release/sentc_api_init ./sentc_api_init

FROM debian:bullseye-slim

ENV SERVER_PORT=3002
ENV SERVER_HOST=0.0.0.0

COPY --from=builder /app/server_api_all /
COPY --from=builder /app/sentc_api_init /
COPY --from=builder /app/dist /dist/
COPY --from=builder /app/db /db/

EXPOSE 3002

CMD ["/server_api_all"]