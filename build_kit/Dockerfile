# ---------- 1. Build stage ----------------------------------------------------
FROM ghcr.io/make87/rust:1-bookworm AS build-image

# + clang toolchain + OpenCV dev libs
RUN apt-get update && apt-get install -y --no-install-suggests --no-install-recommends \
        clang                \
        libclang-dev         \
        llvm-dev             \
        jq build-essential cmake nasm pkg-config \
        libopencv-dev        \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml .
COPY src ./src

RUN cargo build --release

# Determine binary name and move it to /main
RUN binary_name=$(cargo metadata --format-version=1 --no-deps \
        | jq -r '.packages[].targets[] | select(.kind[]=="bin") | .name') \
    && echo "Binary name: $binary_name" \
    && cp target/release/"$binary_name" /main

# Copy ONLY the shared libraries that /main really needs
RUN mkdir /deps \
    && ldd /main | awk '{if ($3 ~ /^\//) print $3}' \
    | xargs -I '{}' cp --parents '{}' /deps

# ---------- 2. Distroless runtime stage --------------------------------------
FROM ghcr.io/make87/cc-debian12:latest

WORKDIR /app

# Binary
COPY --from=build-image /main /app/main
# Exact set of shared objects (keeps the image tiny)
COPY --from=build-image /deps/ /


CMD ["/app/main"]
