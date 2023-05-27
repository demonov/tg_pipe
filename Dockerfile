# Build stage: create the binary
FROM rust:1.69.0 as builder
RUN apt-get update && apt install -y openssl
WORKDIR /usr/src/build

COPY . .

RUN cargo build --release

# Second stage: create the final image
FROM ubuntu:22.04
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/build/target/release/tg_pipe .

# Copy any additional files generated by build.rs
COPY --from=builder /usr/src/build/target/release/build/tg_pipe-*/out/ .

# Run the application
CMD ["./tg_pipe"]
