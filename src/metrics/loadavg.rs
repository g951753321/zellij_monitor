/// Parse `/proc/loadavg` and return `(load_1m, load_5m, load_15m)`.
///
/// Example `/proc/loadavg` content:
/// ```text
/// 0.45 0.52 0.61 2/456 12345
/// ```
pub fn parse_loadavg(content: &str) -> (f32, f32, f32) {
    let mut parts = content.split_whitespace();
    let l1 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let l5 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let l15 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    (l1, l5, l15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_loadavg() {
        let (l1, l5, l15) = parse_loadavg("0.45 0.52 0.61 2/456 12345\n");
        assert!((l1 - 0.45).abs() < 1e-4);
        assert!((l5 - 0.52).abs() < 1e-4);
        assert!((l15 - 0.61).abs() < 1e-4);
    }

    #[test]
    fn parses_high_load() {
        let (l1, l5, l15) = parse_loadavg("12.34 9.01 6.78 10/200 9999\n");
        assert!((l1 - 12.34).abs() < 1e-2);
        assert!((l5 - 9.01).abs() < 1e-2);
        assert!((l15 - 6.78).abs() < 1e-2);
    }

    #[test]
    fn empty_input_returns_zeros() {
        let (l1, l5, l15) = parse_loadavg("");
        assert_eq!(l1, 0.0);
        assert_eq!(l5, 0.0);
        assert_eq!(l15, 0.0);
    }
}
