# Stage 1: compile
FROM rust:latest-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build

# Cache dependencies before copying source
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# Create non-root user
RUN echo "appuser:x:1000:1000::/:" > /passwd && \
    echo "appgroup:x:1000:" > /group

# Stage 2: minimal runtime image
FROM scratch
COPY --from=builder /build/target/release/oko /usr/local/bin/oko
COPY --from=builder /passwd /etc/passwd
COPY --from=builder /group /etc/group
USER appuser
ENTRYPOINT ["/usr/local/bin/oko"]
