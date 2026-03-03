#!/usr/bin/env bash
set -euo pipefail

WASM_TARGET="wasm32-wasip1"
PLUGIN_NAME="zellij_monitor"
PLUGIN_DIR="${HOME}/.config/zellij/plugins"

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

# ── 3. Install ─────────────────────────────────────────────────────────────────
echo "→ Installing to ${PLUGIN_DIR}/"
mkdir -p "${PLUGIN_DIR}"
cp "${WASM_PATH}" "${PLUGIN_DIR}/${PLUGIN_NAME}.wasm"
echo "  Done: ${PLUGIN_DIR}/${PLUGIN_NAME}.wasm"

# ── 4. Clear Zellij plugin cache so the new WASM isn't shadowed ────────────────
echo "→ Clearing Zellij plugin cache (~/.cache/zellij)"
rm -rf ~/.cache/zellij

# ── 5. Kill any stale Zellij server processes ─────────────────────────────────
echo "→ Stopping any running Zellij servers"
pkill -f "zellij --server" 2>/dev/null && echo "  Stopped stale servers." || echo "  No running servers found."

# ── 6. Print KDL snippet ───────────────────────────────────────────────────────
cat <<'EOF'

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Add the following pane to the bottom of your Zellij layout (KDL):

    default_tab_template {
        children

        pane size=1 borderless=true {
            plugin location="file:~/.config/zellij/plugins/zellij_monitor.wasm" {
                // Toggle metrics on/off
                show_cpu        "true"
                show_memory     "true"
                show_disk       "true"
                show_network    "true"
                show_loadavg    "true"

                // Refresh every N seconds (minimum 1)
                refresh_interval "5"

                // Disk path to monitor
                disk_path       "/"

                // Network interface to monitor ("all" = sum non-loopback)
                network_interface "all"

                // Warning thresholds (turn yellow near, red at/above)
                cpu_warn_pct    "80"
                mem_warn_pct    "80"
                disk_warn_pct   "80"
            }
        }
    }

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EOF
