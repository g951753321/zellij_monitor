/// Holds the previous `/proc/net/dev` sample for rate calculation.
#[derive(Debug, Clone)]
pub struct NetworkState {
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    /// True after the first sample has been recorded.
    initialized: bool,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            prev_rx_bytes: 0,
            prev_tx_bytes: 0,
            initialized: false,
        }
    }
}

/// Parse a single data line from `/proc/net/dev`.
/// Returns `(iface_name, rx_bytes, tx_bytes)` or `None` on parse failure.
pub fn parse_netdev_line(line: &str) -> Option<(&str, u64, u64)> {
    let (iface_raw, rest) = line.split_once(':')?;
    let iface = iface_raw.trim();
    let mut nums = rest.split_whitespace();
    let rx: u64 = nums.next()?.parse().ok()?;
    // tx is the 9th field (index 8 after rx)
    let tx: u64 = nums.nth(7)?.parse().ok()?;
    Some((iface, rx, tx))
}

/// Sum rx/tx bytes across the given interface selection from `/proc/net/dev`.
/// `interface` can be:
/// - `"all"` → sum all non-loopback interfaces
/// - a specific interface name like `"eth0"` or `"wlan0"`
pub fn sum_bytes(proc_net_dev: &str, interface: &str) -> (u64, u64) {
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    for line in proc_net_dev.lines().skip(2) {
        if let Some((iface, rx, tx)) = parse_netdev_line(line) {
            let include = if interface == "all" {
                iface != "lo"
            } else {
                iface == interface
            };
            if include {
                total_rx = total_rx.saturating_add(rx);
                total_tx = total_tx.saturating_add(tx);
            }
        }
    }
    (total_rx, total_tx)
}

impl NetworkState {
    /// Feed `/proc/net/dev` content and return `(rx_kbps, tx_kbps)`.
    /// `elapsed_s` is the seconds since the last call (from Zellij's `Timer` event).
    /// Returns `(0.0, 0.0)` on the first call (no previous sample).
    pub fn update(&mut self, proc_net_dev: &str, interface: &str, elapsed_s: f64) -> (f64, f64) {
        let (rx_bytes, tx_bytes) = sum_bytes(proc_net_dev, interface);

        let (rx_kbps, tx_kbps) = if self.initialized && elapsed_s > 0.0 {
            let rx_delta = rx_bytes.saturating_sub(self.prev_rx_bytes) as f64;
            let tx_delta = tx_bytes.saturating_sub(self.prev_tx_bytes) as f64;
            (rx_delta / 1024.0 / elapsed_s, tx_delta / 1024.0 / elapsed_s)
        } else {
            (0.0, 0.0)
        };

        self.prev_rx_bytes = rx_bytes;
        self.prev_tx_bytes = tx_bytes;
        self.initialized = true;

        (rx_kbps, tx_kbps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo:  123456     100    0    0    0     0          0         0   123456     100    0    0    0     0       0          0
  eth0: 1048576    1000    0    0    0     0          0         0   524288     500    0    0    0     0       0          0
  wlan0:  512000     400    0    0    0     0          0         0   256000     200    0    0    0     0       0          0
";

    #[test]
    fn parse_netdev_line_extracts_rx_tx() {
        let line = "  eth0: 1048576    1000    0    0    0     0          0         0   524288     500    0    0    0     0       0          0";
        let (iface, rx, tx) = parse_netdev_line(line).unwrap();
        assert_eq!(iface, "eth0");
        assert_eq!(rx, 1048576);
        assert_eq!(tx, 524288);
    }

    #[test]
    fn sum_bytes_all_excludes_loopback() {
        let (rx, tx) = sum_bytes(SAMPLE, "all");
        assert_eq!(rx, 1048576 + 512000);
        assert_eq!(tx, 524288 + 256000);
    }

    #[test]
    fn sum_bytes_specific_interface() {
        let (rx, tx) = sum_bytes(SAMPLE, "eth0");
        assert_eq!(rx, 1048576);
        assert_eq!(tx, 524288);
    }

    #[test]
    fn sum_bytes_unknown_interface_returns_zero() {
        let (rx, tx) = sum_bytes(SAMPLE, "tun0");
        assert_eq!(rx, 0);
        assert_eq!(tx, 0);
    }

    #[test]
    fn first_update_returns_zero_rates() {
        let mut state = NetworkState::default();
        let (rx, tx) = state.update(SAMPLE, "all", 0.0);
        assert_eq!(rx, 0.0);
        assert_eq!(tx, 0.0);
        // State should be seeded now
        assert!(state.initialized);
    }
}
