/// Parse the output of `df -BM <path>` and return `(used_pct, avail_mib)`.
///
/// Example `df -BM /` output:
/// ```text
/// Filesystem     1M-blocks  Used Available Use% Mounted on
/// /dev/sda1         98304M 45000M     48000M  49% /
/// ```
pub fn parse_df_output(output: &str) -> (u8, u64) {
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Typical columns: Filesystem 1M-blocks Used Available Use% Mounted
        if parts.len() >= 5 {
            let used_pct = parts[4]
                .trim_end_matches('%')
                .parse::<u8>()
                .unwrap_or(0);
            let avail_mib = parts[3]
                .trim_end_matches('M')
                .parse::<u64>()
                .unwrap_or(0);
            return (used_pct, avail_mib);
        }
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Filesystem     1M-blocks   Used Available Use% Mounted on
/dev/sda1         98304M  45000M    48000M  49% /
";

    #[test]
    fn parses_used_pct_and_avail() {
        let (pct, avail) = parse_df_output(SAMPLE);
        assert_eq!(pct, 49);
        assert_eq!(avail, 48000);
    }

    #[test]
    fn handles_100_percent_disk() {
        let input = "Filesystem 1M-blocks Used Available Use% Mounted\n/dev/sdb1 1000M 1000M 0M 100% /data\n";
        let (pct, avail) = parse_df_output(input);
        assert_eq!(pct, 100);
        assert_eq!(avail, 0);
    }

    #[test]
    fn empty_output_returns_zeros() {
        let (pct, avail) = parse_df_output("");
        assert_eq!(pct, 0);
        assert_eq!(avail, 0);
    }

    #[test]
    fn header_only_returns_zeros() {
        let (pct, avail) = parse_df_output("Filesystem 1M-blocks Used Available Use% Mounted\n");
        assert_eq!(pct, 0);
        assert_eq!(avail, 0);
    }
}
