#!/bin/bash
set -e

echo "=== Step 1: Compiling Immutable Target via Local BuildKit ==="
docker build -t research-thin-server:latest .

echo "=== Step 2: Compressing Target Layer Matrix to Tarball ==="
docker save research-thin-server:latest | gzip > research-thin-server.tar.gz

echo "=== Step 3: Preparing Target Workspace Structure over Proxy Jump ==="
ssh docker-vm "mkdir -p /opt/research-thin-server"

echo "=== Step 4: Shipping Orchestration Core and Binary Layers ==="
scp research-thin-server.tar.gz docker-compose.yml docker-vm:/opt/research-thin-server/

echo "=== Step 5: Hot Reloading Remote Container Stack ==="
ssh docker-vm "
  cd /opt/research-thin-server && \
  gzip -dc research-thin-server.tar.gz | docker load && \
  rm research-thin-server.tar.gz && \
  docker compose up -d
"

echo "=== Step 6: Cleaning Local Temp Archives ==="
rm research-thin-server.tar.gz

echo "=== Live Deployment Completed Successfully ==="
