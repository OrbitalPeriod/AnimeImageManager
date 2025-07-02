FROM python:3.11-slim AS base

# Install Rust
RUN apt-get update && apt-get install -y curl build-essential pkg-config libssl-dev && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Build TagManager
WORKDIR /build/tag_manager
COPY tag_manager ./tag_manager
COPY .env_docker ./tag_manager/.env
RUN cd tag_manager && cargo build --release

# Build TagApi
WORKDIR /build/tag_api
COPY tag_api ./tag_api
COPY .env_docker ./tag_api/.env
RUN cd tag_api && cargo build --release

FROM python:3.11-slim

# Setup Python service
WORKDIR /app
COPY TagService/ ./TagService
COPY TagService/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

WORKDIR /app 
COPY PixivDownloader/ ./PixivDownloader 
COPY PixivDownloader/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy Rust binaries
COPY --from=base /build/tag_manager/tag_manager/target/release/tag_manager /usr/local/bin/
COPY --from=base /build/tag_api/tag_api/target/release/tag_api /usr/local/bin/


# Add entrypoint script
COPY run_all.sh /run_all.sh
RUN chmod +x /run_all.sh

EXPOSE 8080

CMD ["/run_all.sh"]
