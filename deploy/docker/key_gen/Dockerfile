# syntax=docker/dockerfile:experimental
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/app/target \
	cargo build --release && \
    # Copy executable out of the cache so it is available in the final image.
    cp target/release/sentc_api_key_gen ./sentc_api_key_gen

FROM debian:bullseye-slim

COPY --from=builder /app/sentc_api_key_gen /

CMD ["/sentc_api_key_gen"]