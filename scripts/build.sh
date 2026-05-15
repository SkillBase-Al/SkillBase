#!/usr/bin/env bash
set -euo pipefail

echo "=== SkillBase Build ==="

# Build server
echo "[1/3] Building server..."
cd packages/server
cargo build --release
cd ../..

# Build client (Tauri desktop app)
echo "[2/3] Building Tauri desktop app..."
cd packages/client
npm install
npm run tauri build
cd ../..

# Build Docker image
echo "[3/3] Building Docker image..."
docker compose -f packages/server/docker-compose.yml build

echo ""
echo "=== Build complete ==="
echo "  Server binary: packages/server/target/release/skillbase-server"
echo "  Tauri bundle:  packages/client/src-tauri/target/release/bundle/"
echo "  Docker image:  skillbase-server:latest"
