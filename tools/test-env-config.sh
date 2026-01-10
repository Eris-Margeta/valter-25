#!/bin/bash

# =========================================================================
# SKRIPTA ZA AUTOMATIZIRANO TESTIRANJE KONFIGURACIJE
# v1.2 - Ispravno prosljeđivanje varijabli
# =========================================================================

set -e

# --- KONFIGURACIJA ---
TEST_LOG_FILE="valter-test.log"
TEST_BINARY_PATH="./target/debug/valter"
DEV_PORT=$(grep 'port:' valter.dev.config | sed 's/port: //g' | tr -d ' ')

# --- FUNKCIJE ---
kill_port() {
  echo "🔪 Gašenje procesa na portu $DEV_PORT..."
  lsof -ti :$DEV_PORT | xargs kill -9 >/dev/null 2>&1 || true
  sleep 0.5
}

run_test() {
  local test_name="$1"
  local expected_status="$2"
  # Ovdje primamo varijable kao niz riječi
  local extra_vars=("${@:3}")

  echo ""
  echo "🧪 Pokretanje testa: $test_name..."

  kill_port
  # Brišemo log PRIJE pokretanja da ne uhvatimo stare greške
  >"$TEST_LOG_FILE"
  echo -e "\n--- TEST: $test_name ---" >>$TEST_LOG_FILE

  (
    # ISPRAVAK: Prosljeđujemo varijable direktno, bez navodnika
    env -i PATH="$PATH" HOME="$HOME" VALTER_TEST_MODE="true" "${extra_vars[@]}" \
      $TEST_BINARY_PATH run >>$TEST_LOG_FILE 2>&1
  ) &
  PID=$!

  sleep 2

  if kill -0 $PID >/dev/null 2>&1; then
    kill $PID
  fi
  wait $PID || true

  if ! grep -q "$expected_status" "$TEST_LOG_FILE"; then
    echo "❌ TEST FAILED: $test_name"
    echo "   Očekivani status NIJE pronađen: '$expected_status'"
    exit 1
  fi

  if grep -q "API Fatal" "$TEST_LOG_FILE"; then
    echo "❌ TEST FAILED: $test_name"
    echo "   Pronađena je neočekivana FATALNA GREŠKA u logu!"
    exit 1
  fi

  echo "✅ PASS: $test_name"
}

# --- GLAVNI TOK IZVRŠAVANJA ---
echo "🚀 Započinjem testiranje konfiguracije..."
cd "$(dirname "$0")/.."
rm -f $TEST_LOG_FILE
touch $TEST_LOG_FILE

echo "🏗️  Kompajliranje testne binarne datoteke..."
ENV_VARIABLES_HIERARCHY_ENABLE_OVERRIDES_FROM_SYSTEM_DURING_RUNTIME="true" \
  cargo build --manifest-path core/Cargo.toml

# Testovi...
run_test "RUNTIME-ERROR (nedostaju varijable)" \
  "Environment Config Status: RuntimeError"

run_test "RUNTIME-OVERRIDE (sve varijable postavljene)" \
  "Environment Config Status: Runtime" \
  VALTER_PROVIDER="runtime-gemini" VALTER_GEMINI_API_KEY="runtime-key" VALTER_MODEL="runtime-model" VALTER_RPM="999" VALTER_SEARCH_API_KEY="runtime-search-key" VALTER_SEARCH_CX="runtime-cx"

run_test "FALLBACK-VIA-IGNORE ('ignore' zastavica)" \
  "Environment Config Status: CompileTimeIgnored" \
  VALTER_IGNORE_ENV_DURING_RUNTIME="true"

kill_port
echo ""
echo "🎉 Svi testovi konfiguracije su USPJEŠNO PROŠLI."
