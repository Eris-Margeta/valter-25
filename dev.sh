#!/bin/bash

# Funkcija za gašenje svih procesa kad stisnemo Ctrl+C
cleanup() {
  echo "Shutting down VALTER..."
  # Ubija sve child procese u ovoj grupi procesa
  trap - SIGTERM && kill -- -$$
}

trap cleanup SIGINT SIGTERM EXIT

echo "---------------------------------------------------"
echo "  VALTER ERP - DEV ENVIRONMENT"
echo "---------------------------------------------------"

# 1. Provjera Configa
if [ ! -f "valter.config" ]; then
  echo "⚠️  valter.config not found! Copying example..."
  cp valter.config.example valter.config
fi

# 2. Pokretanje Backenda (Core)
echo "[CORE] Starting Daemon (Port 8000)..."
cd core
# Koristimo cargo run u backgroundu (&)
cargo run >../valter.log 2>&1 &
CORE_PID=$!
cd ..

# Čekamo malo da se backend digne prije nego palimo frontend
sleep 2
echo "[CORE] Logs are streaming to ./valter.log"

# 3. Pokretanje Web Stranice (Docs Sync) - Opcionalno
# echo "[DOCS] Syncing Documentation..."
# cd website
# node scripts/sync-docs.js
# cd ..

# 4. Pokretanje Frontenda (Dashboard)
echo "[DASH] Starting Interface (Port 5173)..."
cd dashboard
# pnpm dev obično ima lijepe boje, pa ga nećemo pipati u log file, neka piše u terminal
pnpm dev &
DASH_PID=$!
cd ..

echo "---------------------------------------------------"
echo "SYSTEM ONLINE"
echo ">> Dashboard: http://localhost:5173"
echo ">> GraphiQL:  http://localhost:8000/graphql"
echo "---------------------------------------------------"
echo "Press Ctrl+C to stop all services."

# Čekaj zauvijek (dok ne stisnemo Ctrl+C)
wait
