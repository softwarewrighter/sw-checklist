#!/bin/bash
# Run the sw-checklist CLI binary
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/../components/checklist-cli/target/release/sw-checklist"

if [ ! -x "$BINARY" ]; then
    echo "Binary not found. Run ./scripts/build-all.sh first." >&2
    exit 1
fi

exec "$BINARY" "$@"
