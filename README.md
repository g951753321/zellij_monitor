# zellij-monitor

A [Zellij](https://zellij.dev) status-bar plugin that displays live system
metrics at the bottom of your terminal.

```
 CPU  42%  │  MEM 3.1/16.0 GiB  │  TEMP  52°C  │  DISK  49% 48.0GiB free  │  NET ↓1.2MB/s ↑0.3MB/s  │  LOAD 0.45 0.52 0.61
```

## Features

| Metric | Source | Notes |
|--------|--------|-------|
| CPU usage % | `/proc/stat` (delta) | All cores combined |
| Memory used / total | `/proc/meminfo` | Uses `MemAvailable` for accuracy |
| CPU temperature | `/sys/class/thermal/thermal_zone*/temp` | Average across all thermal zones |
| Disk used % + free | `df -BM` | Configurable mount path |
| Network RX/TX rate | `/proc/net/dev` (delta) | Specific iface or sum-all |
| Load average 1/5/15 m | `/proc/loadavg` | |

## Quick Start

### Install

```bash
chmod +x install.sh
./install.sh
```

`install.sh` will:
1. Install the `wasm32-wasip1` Rust target if missing
2. Build the release WASM binary
3. Copy it to `~/.config/zellij/plugins/`
4. Print the KDL snippet to add to your layout

### Add to a Zellij Layout

In your layout KDL file, add the plugin pane at the bottom of
`default_tab_template`:

```kdl
default_tab_template {
    children

    pane size=1 borderless=true {
        plugin location="file:~/.config/zellij/plugins/zellij_monitor.wasm" {
            show_cpu        "true"
            show_memory     "true"
            show_cpu_temp   "true"
            show_disk       "true"
            show_network    "true"
            show_loadavg    "true"
            refresh_interval "5"
            disk_path       "/"
            network_interface "all"
            cpu_warn_pct    "80"
            mem_warn_pct    "80"
            disk_warn_pct   "80"
            cpu_temp_warn   "80"
        }
    }
}
```

### Dev / Live Reload

Build (debug) and launch Zellij with the dev layout:

```bash
cargo build
zellij --layout zellij.kdl
```

## Configuration Reference

All keys are optional. Unset keys use the defaults shown below.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `show_cpu` | `"true"` / `"false"` | `"true"` | Show CPU usage |
| `show_memory` | `"true"` / `"false"` | `"true"` | Show memory usage |
| `show_disk` | `"true"` / `"false"` | `"true"` | Show disk usage |
| `show_network` | `"true"` / `"false"` | `"true"` | Show network I/O |
| `show_loadavg` | `"true"` / `"false"` | `"true"` | Show load average |
| `show_cpu_temp` | `"true"` / `"false"` | `"true"` | Show CPU temperature (avg across thermal zones) |
| `refresh_interval` | integer (seconds) | `"5"` | Polling interval (min 1 s) |
| `disk_path` | path string | `"/"` | Mount point to monitor |
| `network_interface` | interface name or `"all"` | `"all"` | Interface to track; `"all"` sums non-loopback |
| `cpu_warn_pct` | 0–100 | `"80"` | CPU % threshold for yellow → red |
| `mem_warn_pct` | 0–100 | `"80"` | Memory % threshold |
| `disk_warn_pct` | 0–100 | `"80"` | Disk % threshold |
| `cpu_temp_warn` | 0–100 | `"80"` | CPU temp °C threshold |

### Colour Coding

- **Green** — value is more than 10 % below the warning threshold
- **Yellow** — value is within 10 % of the warning threshold
- **Red** — value is at or above the warning threshold

## Building from Source

```bash
# Install target (once)
rustup target add wasm32-wasip1

# Debug build (faster, larger)
cargo build

# Release build (optimised, smaller)
cargo build --release
```

## Running Tests

Tests run against the native target (the WASM target cannot run tests directly):

```bash
cargo test --target x86_64-unknown-linux-gnu
```

## Permissions

The plugin requests the following Zellij permissions at startup:

| Permission | Why |
|------------|-----|
| `FullHdAccess` | Read `/proc/stat`, `/proc/meminfo`, `/proc/loadavg`, `/proc/net/dev`, `/sys/class/thermal/thermal_zone*/temp` |
| `RunCommands` | Execute `df -BM <path>` for disk usage |
| `ReadApplicationState` | Receive timer and command-result events |

Zellij will prompt you to grant these on first launch.

## Uninstall

```bash
rm ~/.config/zellij/plugins/zellij_monitor.wasm
```

Remove the plugin pane from your layout KDL file.
