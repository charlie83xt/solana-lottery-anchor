# Use an official Rust image as the base
FROM rust:1.65

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Solana CLI
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.10/install)"

# Install Anchor CLI
RUN cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
