#!/bin/bash

# Funkcija za gašenje procesa na izlazu
cleanup() {
  echo "Shutting down Strata..."
  # Ubija sve child procese (cargo i vite)
  pkill -P $$
  exit
}

trap cleanup SIGINT SIGTERM

echo "---------------------------------------------------"
echo "  STRATA ENGINE - SYSTEM STARTUP (PNPM MODE)"
echo "---------------------------------------------------"

# 1. Provjera Configa
if [ ! -f "strata.config" ]; then
  echo "Error: strata.config not found!"
  exit 1
fi

# 2. Pokretanje Backenda (Rust)
echo "[1/3] Launching Daemon (Port 8000)..."
# Brišemo stari log da bude čist
rm -f strata.log
cargo run >strata.log 2>&1 &
BACKEND_PID=$!

# Čekamo da se backend digne
echo "Waiting for Daemon..."
sleep 2
if ! ps -p $BACKEND_PID >/dev/null; then
  echo "Error: Daemon failed to start. Check strata.log."
  exit 1
fi

# 3. Priprema Frontenda (Hard Reset & PNPM)
echo "[2/3] Preparing Frontend (Hard Cache Reset)..."
cd web

# BRISANJE SVEGA: node_modules, cache, lock fileovi
rm -rf node_modules pnpm-lock.yaml package-lock.json .vite dist

# Instalacija putem PNPM
echo "Installing Dependencies via pnpm..."
pnpm install >/dev/null 2>&1

if [ $? -ne 0 ]; then
  echo "Error: pnpm install failed. Do you have pnpm installed? (npm install -g pnpm)"
  kill $BACKEND_PID
  exit 1
fi

# 4. Pokretanje Frontenda
echo "[3/3] Launching Interface (Port 5173)..."
# Pišemo log u ROOT direktorij (../vite.log)
rm -f ../vite.log
pnpm dev >../vite.log 2>&1 &
FRONTEND_PID=$!

# Povratak u root
cd ..

echo "---------------------------------------------------"
echo "SYSTEM ONLINE"
echo ">> Dashboard: http://localhost:5173"
echo ">> GraphiQL:  http://localhost:8000/graphql"
echo "---------------------------------------------------"
echo "Logs are being written to:"
echo "  - Backend:  ./strata.log"
echo "  - Frontend: ./vite.log"
echo "---------------------------------------------------"
echo "Press Ctrl+C to stop."

# Čekaj zauvijek
wait
