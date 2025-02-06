#!/usr/bin/env sh

SCRIPT_DIR=$(dirname -- "$0")

if ! command -v sea-orm-cli &>/dev/null; then
	echo 'FATAL ERROR: A "sea-orm-cli" binary must be available.'
	echo 'Use `cargo install sea-orm-cli` or equivalent, and then re-run this script.'
	exit 1
fi

sea-orm-cli generate entity --lib --output-dir "$SCRIPT_DIR/src"
