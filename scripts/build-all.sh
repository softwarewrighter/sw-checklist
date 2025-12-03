#!/bin/bash
# Build all components in dependency order
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building all components..."

# Component build order (dependencies first):
# Layer 1: No dependencies
# Layer 2: Depends on Layer 1
# Layer 3: Handler components depend on Layer 1+2 + handler-trait
# Layer 4: CLI depends on all above

echo ""
echo "=== Building checklist-model ==="
cd "$REPO_ROOT/components/checklist-model"
cargo build --release

echo ""
echo "=== Building checklist-discovery ==="
cd "$REPO_ROOT/components/checklist-discovery"
cargo build --release

echo ""
echo "=== Building checklist-handler-trait ==="
cd "$REPO_ROOT/components/checklist-handler-trait"
cargo build --release

echo ""
echo "=== Building checklist-handler-cargo ==="
cd "$REPO_ROOT/components/checklist-handler-cargo"
cargo build --release

echo ""
echo "=== Building checklist-handler-clap ==="
cd "$REPO_ROOT/components/checklist-handler-clap"
cargo build --release

echo ""
echo "=== Building checklist-handler-wasm ==="
cd "$REPO_ROOT/components/checklist-handler-wasm"
cargo build --release

echo ""
echo "=== Building checklist-handler-modularity ==="
cd "$REPO_ROOT/components/checklist-handler-modularity"
cargo build --release

echo ""
echo "=== Building checklist-cli ==="
cd "$REPO_ROOT/components/checklist-cli"
cargo build --release

echo ""
echo "Build complete!"
echo "Binary available at: $REPO_ROOT/components/checklist-cli/target/release/sw-checklist"
