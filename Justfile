# VALTER ERP - JUSTFILE
# Commands for Development, Installation, Maintenance, and Release.

# Default: List available commands
default:
    @just --list

# --- DEVELOPMENT ---

# Run the full stack in Dev Mode (Monorepo root)
dev:
    @echo "🚀 Starting VALTER DEV Environment..."
    @# Run Backend in background, then Frontend. Trap SIGINT to kill backend on Ctrl+C.
    @trap 'kill %1' SIGINT; \
    (cd core && cargo run) & \
    (cd dashboard && pnpm dev)

# Clean build artifacts and temporary files
clean:
    @echo "🧹 Cleaning up..."
    rm -rf target core/target
    rm -rf dashboard/dist dashboard/.vite dashboard/node_modules
    rm -f valter.db valter.log

# --- RELEASE ---

# Create a new release (Tag & Push). Usage: just release v0.1.0
release version:
    @echo "🚀 Preparing release {{version}}..."
    @if [ -z "{{version}}" ]; then echo "❌ Error: Version required. Usage: just release v0.1.0"; exit 1; fi
    @if [ -n "$(git status --porcelain)" ]; then echo "❌ Error: Git is dirty. Commit changes first."; exit 1; fi
    @echo "🏷️  Tagging version {{version}}..."
    git tag -a {{version}} -m "Release {{version}}"
    @echo "⬆️  Pushing tag to GitHub..."
    git push origin {{version}}
    @echo "✅ Done! GitHub Actions will now build and publish the release."
    @echo "   Check status here: https://github.com/Eris-Margeta/valter-25/actions"

# --- INSTALLATION (SYSTEM WIDE) ---

# Install Valter to ~/.local/bin and setup ~/.valter config
install:
    @echo "⚠️  WARNING: This will overwrite ~/.valter configuration and binary."
    @echo "   Press Ctrl+C to cancel or Enter to proceed."
    @read _
    
    @echo "📦 Building Release Binary..."
    # Build from root workspace
    cargo build --release
    
    @echo "📂 Creating System Directories (~/.valter)..."
    mkdir -p ~/.valter
    mkdir -p ~/.local/bin
    
    @echo "🚚 Installing Binary..."
    # FIX: Workspace puts artifacts in root target/release
    cp target/release/valter ~/.local/bin/valter
    
    @echo "📝 Installing Default Config..."
    # We copy the example config to the production path
    cp valter.config.example ~/.valter/valter.config
    
    @echo "✅ Installation Complete!"
    @echo "   Run 'valter' to start the daemon."
    @echo "   (Ensure ~/.local/bin is in your PATH)"

# --- MAINTENANCE ---

# Update the binary ONLY (Preserves Data & Config)
update:
    @echo "🔄 Updating Valter Binary..."
    cargo build --release
    cp target/release/valter ~/.local/bin/valter
    @echo "✅ Binary Updated. Run 'valter' to apply migrations."

# Check for database schema changes (Dry Run - Placeholder for future CLI arg)
check-migrations:
    @echo "🔍 Checking migrations..."
    @echo "Note: Currently Valter applies migrations automatically on startup."
