#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

VENV_DIR=".venv"

# create venv if missing
if [ ! -d "$VENV_DIR" ]; then
    echo "Creating Python virtual environment..." >&2
    python -m venv "$VENV_DIR"
fi

# activate venv
source "$VENV_DIR/bin/activate"

# install Python deps if marker is stale
MARKER="$VENV_DIR/.deps_installed"
if [ ! -f "$MARKER" ] || [ requirements.txt -nt "$MARKER" ]; then
    echo "Installing Python dependencies..." >&2
    pip install -r requirements.txt
    touch "$MARKER"
fi

if [ ! -f .env ]; then
    echo "No .env found. Copy .env.example to .env and add your API key(s)." >&2
    exit 1
fi

exec cargo run --release "$@"
