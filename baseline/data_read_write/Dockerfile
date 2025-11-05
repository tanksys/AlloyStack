FROM ghcr.io/openfaas/of-watchdog:0.9.10 as watchdog

WORKDIR /home/rust

# Copy all the sources
# COPY function ./function
# COPY main ./main

ENV CARGO_TARGET_DIR=/home/rust/target
WORKDIR /home/rust/main
# RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.17.3 as runner

# Install packages and add non-root user
RUN apk --no-cache add curl ca-certificates \
    && addgroup -S app && adduser -S -g app app

ENV USER=app

# Copy of-watchdog binary
COPY --from=watchdog /fwatchdog /usr/bin/fwatchdog

# AE: unable to get the binary to start if moved to /home/app
# COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/release/main /usr/bin/
COPY ./data-rw-rust/target/x86_64-unknown-linux-musl/release/main /usr/bin/

WORKDIR /home/app

ENV fprocess="main"
ENV mode="http"
ENV upstream_url="http://127.0.0.1:3000"
ENV RUST_BACKTRACE=1

HEALTHCHECK --interval=3s CMD [ -e /tmp/.lock ] || exit 1

CMD ["fwatchdog"]

