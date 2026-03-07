#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT="$REPO_ROOT/packages/schema-types/src/index.ts"

echo "Generating TypeScript types from Rust core-types..."
cargo run --bin schema-gen > "$OUTPUT"
echo "Wrote $OUTPUT"
