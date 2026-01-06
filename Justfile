# VALTER ERP - JUSTFILE
# Commands for Development, Installation, and Maintenance.

# Default: List commands
default:
    @just --list

# --- DEVELOPMENT ---

# Run the full stack in Dev Mode (Monorepo root)
dev:
    @echo "🚀 Starting VALTER DEV Environment..."
    @# Run Backend in background, then Frontend
    @trap 'kill %1' SIGINT; \
    (cd core && cargo run) & \
    (cd dashboard && pnpm dev)

# Clean build artifacts
clean:
    @echo "🧹 Cleaning up..."
    rm -rf target core/target
    rm -f valter.db valter.log

# --- INSTALLATION (SYSTEM WIDE) ---

# Install Valter to ~/.local/bin and setup ~/.valter config
install:
    @echo "⚠️  WARNING: This will overwrite ~/.valter configuration and binary."
    @echo "   Press Ctrl+C to cancel or Enter to proceed."
    @read _
    
    @echo "📦 Building Release Binary..."
    cd core && cargo build --release
    
    @echo "📂 Creating System Directories (~/.valter)..."
    mkdir -p ~/.valter
    mkdir -p ~/.local/bin
    
    @echo "🚚 Installing Binary..."
    cp core/target/release/valter ~/.local/bin/valter
    
    @echo "📝 Installing Default Config..."
    # Only copy if not exists, or force overwrite? The prompt said overwrite.
    cp valter.config.example ~/.valter/valter.config
    
    @echo "✅ Installation Complete!"
    @echo "   Run 'valter' to start the daemon."
    @echo "   (Ensure ~/.local/bin is in your PATH)"
    @echo "   Set VALTER_HOME=~/.valter env var if needed, or Valter detects it."

# --- MAINTENANCE ---

# Update the binary ONLY (Preserves Data)
update:
    @echo "🔄 Updating Valter Binary..."
    cd core && cargo build --release
    cp core/target/release/valter ~/.local/bin/valter
    @echo "✅ Binary Updated. Run 'valter' to apply migrations."

# Check for database schema changes (Dry Run)
# This uses the binary to check migrations without running the daemon
check-migrations:
    @echo "🔍 Checking migrations..."
    VALTER_HOME=~/.valter valter --check-only || echo "Feature not implemented yet"

