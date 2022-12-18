FROM rust:1.66.0 as builder
WORKDIR /usr/src/lfb-back
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install openssl && apt-get install ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/lfb-back /usr/local/bin/lfb-back
COPY --from=builder /usr/src/lfb-back/.env .env

EXPOSE 8000

CMD ["lfb-back"]