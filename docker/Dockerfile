## -----------------------------------------------------------------------------
## Build
## -----------------------------------------------------------------------------
FROM clux/muslrust:1.48.0-stable as build-stage

WORKDIR "/build"
COPY . .
RUN cargo build --release

## -----------------------------------------------------------------------------
## Package
## -----------------------------------------------------------------------------
FROM alpine:3.9

COPY --from=build-stage "/build/target/x86_64-unknown-linux-musl/release/storage" "/app/storage"

WORKDIR "/app"
ENTRYPOINT ["/app/storage"]
