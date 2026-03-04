#!/usr/bin/env bash
set -euo pipefail

WASM_TARGET="wasm32-wasip1"
PLUGIN_NAME="zellij_monitor"

# ── 0. Resolve Zellij config directory ────────────────────────────────────────
if [[ -n "${ZELLIJ_CONFIG_DIR:-}" ]]; then
    CONFIG_PATH="${ZELLIJ_CONFIG_DIR}"
else
    CONFIG_PATH="${HOME}/.config/zellij"
fi

PLUGIN_DIR="${CONFIG_PATH}/plugins"
LAYOUT_DIR="${CONFIG_PATH}/layouts"

# ── 1. Ensure the wasm32-wasip1 target is installed ────────────────────────────
echo "→ Checking for Rust target: ${WASM_TARGET}"
if ! rustup target list --installed | grep -q "^${WASM_TARGET}"; then
    echo "  Installing ${WASM_TARGET}…"
    rustup target add "${WASM_TARGET}"
else
    echo "  ${WASM_TARGET} already installed."
fi

# ── 2. Build release binary ────────────────────────────────────────────────────
echo "→ Building release WASM…"
cargo build --release

WASM_PATH="target/${WASM_TARGET}/release/${PLUGIN_NAME}.wasm"
if [[ ! -f "${WASM_PATH}" ]]; then
    echo "ERROR: Expected WASM artifact not found at ${WASM_PATH}" >&2
    exit 1
fi

# ── 3. Install plugin ──────────────────────────────────────────────────────────
echo "→ Installing to ${PLUGIN_DIR}/"
mkdir -p "${PLUGIN_DIR}"
cp "${WASM_PATH}" "${PLUGIN_DIR}/${PLUGIN_NAME}.wasm"
echo "  Done: ${PLUGIN_DIR}/${PLUGIN_NAME}.wasm"

# ── 4. Install default layout ─────────────────────────────────────────────────
echo "→ Creating layout at ${LAYOUT_DIR}/default.kdl"
mkdir -p "${LAYOUT_DIR}"
sed "s|file:target/wasm32-wasip1/debug/zellij_monitor.wasm|file:${PLUGIN_DIR}/${PLUGIN_NAME}.wasm|" \
    zellij.kdl > "${LAYOUT_DIR}/default.kdl"
echo "  Done: ${LAYOUT_DIR}/default.kdl"

# ── 5. Clear Zellij plugin cache so the new WASM isn't shadowed ────────────────
echo "→ Clearing Zellij plugin cache (~/.cache/zellij)"
rm -rf ~/.cache/zellij

# ── 6. Kill any stale Zellij server processes ─────────────────────────────────
echo "→ Stopping any running Zellij servers"
pkill -f "zellij --server" 2>/dev/null && echo "  Stopped stale servers." || echo "  No running servers found."
