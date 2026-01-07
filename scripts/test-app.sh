#!/bin/bash
# Skripta za testiranje frontend aplikacije (lint + build)
set -e # Prekini izvršavanje ako bilo koja naredba ne uspije

echo "🚀 Pokrećem testiranje Valter aplikacije..."

# Odredi putanju do 'app' direktorija relativno od lokacije skripte
APP_DIR=$(dirname "$0")/../app

# Provjeri postoji li direktorij
if [ ! -d "$APP_DIR" ]; then
  echo "❌ Greška: 'app' direktorij nije pronađen."
  exit 1
fi

echo "✅ Radim u direktoriju: $APP_DIR"
cd "$APP_DIR"

echo "🔎 Pokrećem lint provjeru..."
pnpm lint

echo "📦 Pokrećem build proces..."
pnpm build

echo "🎉 Testiranje aplikacije (lint & build) je USPJEŠNO završeno."
