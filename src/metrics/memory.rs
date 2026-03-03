/// Parse `/proc/meminfo` and return `(used_mib, total_mib)`.
///
/// `used_mib = total_mib - available_mib`
pub fn parse_meminfo(meminfo: &str) -> (u64, u64) {
    let mut total_kb: u64 = 0;
    let mut avail_kb: u64 = 0;

    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = first_number(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            avail_kb = first_number(rest);
        }
        if total_kb > 0 && avail_kb > 0 {
            break;
        }
    }

    let total_mib = total_kb / 1024;
    let avail_mib = avail_kb / 1024;
    let used_mib = total_mib.saturating_sub(avail_mib);
    (used_mib, total_mib)
}

fn first_number(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
MemTotal:       16384000 kB
MemFree:         2048000 kB
MemAvailable:    8192000 kB
Buffers:          512000 kB
Cached:          4096000 kB
";

    #[test]
    fn parses_used_and_total() {
        let (used, total) = parse_meminfo(SAMPLE);
        // total = 16384000 / 1024 = 16000 MiB
        // avail = 8192000  / 1024 = 8000 MiB
        // used  = 16000 - 8000 = 8000 MiB
        assert_eq!(total, 16000);
        assert_eq!(used, 8000);
    }

    #[test]
    fn empty_input_returns_zeros() {
        let (used, total) = parse_meminfo("");
        assert_eq!(used, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn missing_available_falls_back_to_total() {
        let input = "MemTotal: 4096000 kB\n";
        let (used, total) = parse_meminfo(input);
        assert_eq!(total, 4000);
        assert_eq!(used, 4000); // avail = 0 → used = total
    }
}
