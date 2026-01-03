#!/bin/bash

# Cleanup function to kill background processes on exit
cleanup() {
    echo "Shutting down Strata..."
    kill $(jobs -p) 2>/dev/null
    exit
}

trap cleanup SIGINT SIGTERM

echo "---------------------------------------------------"
echo "  STRATA ENGINE - SYSTEM STARTUP"
echo "---------------------------------------------------"

# Check config
if [ ! -f "strata.config" ]; then
    echo "Error: strata.config not found!"
    exit 1
fi

# Start Backend
echo "[1/2] Launching Daemon (Port 8000)..."
cargo run > strata.log 2>&1 &
BACKEND_PID=$!

# Wait for Backend
echo "Waiting for Daemon..."
sleep 3
if ! ps -p $BACKEND_PID > /dev/null; then
    echo "Error: Daemon failed to start. Check strata.log."
    exit 1
fi

# Start Frontend
echo "[2/2] Launching Interface (Port 5173)..."
cd web
npm run dev > ../vite.log 2>&1 &
FRONTEND_PID=$!

echo "---------------------------------------------------"
echo "SYSTEM ONLINE"
echo ">> Dashboard: http://localhost:5173"
echo ">> GraphiQL:  http://localhost:8000/graphql"
echo "---------------------------------------------------"
echo "Logs are being written to strata.log and vite.log"
echo "Press Ctrl+C to stop."

# Keep script running
wait
