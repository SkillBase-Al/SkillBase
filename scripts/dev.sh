#!/usr/bin/env bash
set -euo pipefail

echo "=== SkillBase Development Environment ==="

# Start server in background if docker-compose is available
if command -v docker &>/dev/null && [ -f "packages/server/docker-compose.yml" ]; then
    echo "[1/4] Starting Docker services (PostgreSQL)..."
    docker compose -f packages/server/docker-compose.yml up -d
fi

# Start the server
echo "[2/4] Starting backend server..."
cd packages/server
cargo run &
SERVER_PID=$!
cd ../..

# Wait for server to be ready
echo "[3/4] Waiting for server..."
for i in $(seq 1 15); do
    if curl -s http://localhost:3000/api/v1/health > /dev/null 2>&1; then
        echo "  Server is ready!"
        break
    fi
    sleep 1
done

# Start Tauri dev
echo "[4/4] Starting Tauri desktop app..."
cd packages/client
npm run tauri dev &
TAURI_PID=$!
cd ../..

echo ""
echo "=== All services started ==="
echo "  Server:  http://localhost:3000"
echo "  Tauri:   Development window should open"
echo ""
echo "Press Ctrl+C to stop all services"

# Cleanup on exit
trap "kill $SERVER_PID $TAURI_PID 2>/dev/null; echo 'Services stopped.'" EXIT
wait
