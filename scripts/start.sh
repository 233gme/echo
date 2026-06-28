#!/bin/bash
# Запуск Echo: Rust + Python backend

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Echo Launcher ==="
echo "Project: $PROJECT_ROOT"

# Check Python venv
if [ ! -d "$PROJECT_ROOT/backend/venv" ]; then
    echo "Creating Python virtual environment..."
    python3 -m venv "$PROJECT_ROOT/backend/venv"
    source "$PROJECT_ROOT/backend/venv/bin/activate"
    pip install --upgrade pip
    pip install -r "$PROJECT_ROOT/backend/requirements.txt"
else
    source "$PROJECT_ROOT/backend/venv/bin/activate"
fi

# Start backend in background
echo "Starting Python backend..."
cd "$PROJECT_ROOT/backend"
uvicorn main:app --host 127.0.0.1 --port 8000 &
BACKEND_PID=$!

# Wait for backend
echo "Waiting for backend..."
sleep 2

# Start Rust app
echo "Starting Rust app..."
cd "$PROJECT_ROOT"
cargo run &
RUST_PID=$!

# Trap to kill both on exit
cleanup() {
    echo "Shutting down..."
    kill $BACKEND_PID 2>/dev/null || true
    kill $RUST_PID 2>/dev/null || true
    exit
}
trap cleanup INT TERM

echo "Both services running. Press Ctrl+C to stop."
wait
