# VALTER ERP - JUSTFILE
# Commands for Development, Installation, Maintenance, and Release.

default:
    @just --list

# --- DEVELOPMENT ---

# Run the Tauri Desktop App (Unified) in Dev Mode
dev:
    @echo "🚀 Starting Valter Desktop App..."
    cd app && pnpm tauri dev

# Run the backend logic independently (Headless Mode)
dev-core:
    @echo "⚙️ Starting Valter Core (Headless)..."
    cargo run --manifest-path core/Cargo.toml -- run

# Run only the Web Frontend (Browser Mode)
dev-web:
    @echo "🌐 Starting Valter Web Dashboard..."
    cd app && pnpm dev

# Clean artifacts
clean:
    @echo "🧹 Cleaning up..."
    rm -rf target core/target app/src-tauri/target
    rm -rf app/dist app/.vite app/node_modules
    rm -f valter.db valter.log valter.pid valter.db-shm valter.db-wal
    rm -f core/src/fs_writer.rs.bk
    cargo clean -p valter

# --- BUILD & RELEASE ---

# Build the Desktop App (DMG/EXE)
build-app:
    @echo "📦 Building Desktop App..."
    cd app && pnpm tauri build

# Build the Headless Server Binary (Legacy/Server Mode)
build-server:
    @echo "🏗️  Building Dashboard Assets..."
    cd app && pnpm install && pnpm build
    @echo "📦 Building Headless Core..."
    cargo build --release --manifest-path core/Cargo.toml

# Release workflow
release version:
    @echo "🚀 Preparing release {{version}}..."
    @if [ -z "{{version}}" ]; then echo "❌ Error: Version required. Usage: just release v0.1.0"; exit 1; fi
    @if [ -n "$(git status --porcelain)" ]; then echo "❌ Error: Git is dirty. Commit changes first."; exit 1; fi
    @echo "📦 Building Assets..."
    cd app && pnpm install && pnpm build
    @echo "🏷️  Tagging & Pushing..."
    git tag -a {{version}} -m "Release {{version}}"
    git push origin {{version}}
    @echo "✅ Done! GitHub Actions will now build and publish the release."

# --- INSTALLATION (SYSTEM WIDE / HEADLESS) ---

install:
    @echo "⚠️  WARNING: This will overwrite ~/.valter configuration and binary."
    @echo "   Press Ctrl+C to cancel or Enter to proceed."
    @read _
    
    @echo "🏗️  Building Dashboard..."
    cd app && pnpm install && pnpm build
    
    @echo "📦 Building Core Binary..."
    cargo build --release --manifest-path core/Cargo.toml
    
    @echo "📂 Creating System Directories (~/.valter)..."
    mkdir -p ~/.valter
    mkdir -p ~/.local/bin
    
    @echo "🚚 Installing Binary..."
    cp target/release/valter ~/.local/bin/valter
    
    @# MACOS SIGNING FIX
    @if [ "$(uname)" = "Darwin" ]; then \
        echo "🍎 macOS detected: Signing binary..."; \
        codesign -s - --force ~/.local/bin/valter; \
    fi
    
    @echo "📝 Installing Default Config..."
    cp valter.config.example ~/.valter/valter.config
    
    @echo "✅ Installation Complete!"
    @echo "   To start: 'valter start'"
    @echo "   Then open: http://localhost:9090 (or configured port)"

# Update the binary without touching config
update:
    @echo "🔄 Updating Valter Binary..."
    @echo "🏗️  Rebuilding Dashboard..."
    cd app && pnpm install && pnpm build
    @echo "📦 Rebuilding Core..."
    cargo build --release --manifest-path core/Cargo.toml
    
    @echo "🚚 Copying binary..."
    cp target/release/valter ~/.local/bin/valter
    
    @if [ "$(uname)" = "Darwin" ]; then \
        echo "🍎 macOS detected: Signing binary..."; \
        codesign -s - --force ~/.local/bin/valter; \
    fi
    
    @echo "✅ Updated. Restart daemon with 'valter stop' then 'valter start'."


# =========================================================================
# FORMATIRANJE KODA
# =========================================================================
# TODO: Dodati formatiranje za React (Prettier), TOML, YAML itd.

# Formatiraj sav Rust kod u projektu prema `rustfmt.toml` pravilima
format-rust:
    @echo "🖌️  Formatiram Rust kod..."
    @cargo fmt --all




# =========================================================================
# LOKALNA VALIDACIJA I CI PROVJERE
# =========================================================================

# GLAVNA NAREDBA: Pokreni SVE provjere, identično kao na CI serveru.
# Ovo je naredba koju treba pokrenuti prije pushanja koda.
test-ci: check-rust lint-rust test-rust test-app
    @echo "\n✅ \033[1;32mSVE CI PROVJERE SU USPJEŠNO PROŠLE!\033[0m"

test-ci-simulation:
    @echo "🤖 Simuliram CI Pipeline..."
    @just check-rust
    @just test-app
    @VALTER_GEMINI_API_KEY="ci-dummy" VALTER_PROVIDER="gemini" just lint-rust
    @VALTER_GEMINI_API_KEY="ci-dummy" VALTER_PROVIDER="gemini" just test-rust

# --- Granularne Provjere (pozivaju se iz `test-ci`) ---

# Provjeri formatiranje Rust koda (ne mijenja fileove)
check-rust:
    @echo "🔍 Provjeravam formatiranje Rust koda (CI mod)..."
    @cargo fmt --all -- --check

# Pokreni strogi linter za Rust, tretira upozorenja kao greške
lint-rust:
    @echo "Lintam Rust kod (CI mod, stroga provjera)..."
    @cargo clippy --workspace --all-targets --all-features -- -D warnings

# Pokreni SVE testove (unit & integration) u cijelom Rust workspaceu
test-rust:
    @echo "🔬 Pokrećem sve Rust testove (unit & integration)..."
    @cargo test --workspace --all-features

# Testira frontend aplikaciju (lint & build)
test-app:
    @echo "Lintam i buildam frontend aplikaciju (CI mod)..."
    @if ./scripts/test-app.sh > app-test.log 2>&1; then \
        echo "✅ Frontend provjere su uspješno prošle."; \
    else \
        echo "\n❌ \033[1;31mPROVJERE ZA FRONTEND NISU USPJELE.\033[0m Provjerite 'app-test.log' za detalje."; \
        exit 1; \
    fi

# =========================================================================
# TESTIRANJE ZA DEBUGIRANJE (Stare naredbe, i dalje korisne)
# =========================================================================

# Pokreni testove samo za 'core' biblioteku
test-rust-core:
    @echo "🔬 Pokrećem testove samo za 'core' crate..."
    @cargo test -p valter

# Pokreni testove i prikaži ispis (println!) iz njih za lakše debugiranje
test-rust-verbose:
    @echo "🔬 Pokrećem sve Rust testove s detaljnim ispisom..."
    @cargo test --workspace -- --nocapture

# Testira logiku konfiguracije s varijablama okruženja
test-config:
    @echo "🧪 Testiram logiku varijabli okruženja..."
    @./scripts/test-env-config.sh


# =========================================================================
# RUST COMPILE PERFORMANCE & MAINTENANCE
# =========================================================================

# Provjeri što se može obrisati (zahtijeva nightly toolchain)
audit-deps:
    @echo "🧹 Tražim nekorištene dependencije..."
    @cargo +nightly udeps

# Provjeri sigurnosne ranjivosti
audit-sec:
    @echo "🛡️  Skeniram sigurnosne propuste..."
    @cargo audit

# Analiziraj što zauzima najviše mjesta u binarnoj datoteci
audit-bloat:
    @echo "🐘 Analiziram veličinu binarne datoteke..."
    @cargo bloat --release --crates -n 10

# Ažuriraj sve biblioteke na najnovije (safe) verzije
update-deps:
    @echo "⬆️  Ažuriram Rust dependencije..."
    @cargo update
    @echo "✅ Gotovo. Pokreni 'just test-ci' za provjeru."

upgrade-deps:
    @echo "⬆️  Ažuriram Rust dependencije..."
    @cargo upgrade
