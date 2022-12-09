FROM rust:alpine AS builder

WORKDIR /build

RUN apk add --no-cache musl-dev

COPY . .

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/cargo \
    CARGO_HOME=/cargo cargo build --locked --release \
    && strip target/release/distodon -o app


FROM scratch

LABEL org.opencontainers.image.source="https://github.com/Defelo/distodon"

ENV RUST_LOG=info

COPY --from=builder /build/app /

ENTRYPOINT ["/app"]
