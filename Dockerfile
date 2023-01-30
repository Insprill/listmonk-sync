####################################################################################################
## Builder
####################################################################################################
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /listmonk-sync

COPY . .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM alpine:latest

# Copy our build
COPY --from=builder /listmonk-sync/target/x86_64-unknown-linux-musl/release/listmonk-sync /usr/local/bin/listmonk-sync

# Use an unprivileged user
RUN adduser --home /nonexistent --no-create-home --disabled-password listmonk-sync
USER listmonk-sync

CMD ["listmonk-sync"]
