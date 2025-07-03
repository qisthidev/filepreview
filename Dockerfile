# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create src directory and dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    imagemagick \
    ffmpeg \
    unoconv \
    curl \
    libreoffice \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Configure ImageMagick policy to allow PDF processing
RUN sed -i 's/<policy domain="coder" rights="none" pattern="PDF" \/>/<policy domain="coder" rights="read|write" pattern="PDF" \/>/g' /etc/ImageMagick-6/policy.xml

# Create app user
RUN useradd -m -s /bin/bash app

# Create app directory
WORKDIR /app

# Copy binary from build stage
COPY --from=builder /app/target/release/filepreview-rust /usr/local/bin/filepreview-rust

# Create directories for file storage
RUN mkdir -p /tmp/previews && chown app:app /tmp/previews

# Switch to app user
USER app

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Start the application
CMD ["filepreview-rust"]