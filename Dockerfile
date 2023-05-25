# Build stage: create the binary
FROM rust:1.69.0 as builder
WORKDIR /usr/src/build
RUN rustup target add x86_64-unknown-linux-musl

COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl

# Second stage: create the final image
FROM alpine:3.18.0
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/build/target/release/tg_pipe .

# Copy any additional files generated by build.rs
COPY --from=builder /usr/src/build/target/release/build/tg_pipe-*/out/ .

# Run the application
CMD ["./tg_pipe"]
