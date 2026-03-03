/// Tracks the previous `/proc/stat` CPU tick counters so we can compute
/// a delta percentage on each update.
#[derive(Debug, Clone, Default)]
pub struct CpuState {
    prev_active: u64,
    prev_total: u64,
    /// `false` on the very first call — seed counters and return 0.0.
    initialized: bool,
}

/// Parse the first line of `/proc/stat` into (user, nice, system, idle,
/// iowait, irq, softirq, steal) and return the CPU usage percentage since
/// the last call.  Returns `0.0` on the very first call (no previous sample).
pub fn parse_cpu_stat(line: &str) -> Option<(u64, u64, u64, u64, u64, u64, u64, u64)> {
    let mut parts = line.split_whitespace();
    let tag = parts.next()?;
    if !tag.starts_with("cpu") {
        return None;
    }
    let vals: Vec<u64> = parts
        .take(8)
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    if vals.len() < 8 {
        return None;
    }
    Some((vals[0], vals[1], vals[2], vals[3], vals[4], vals[5], vals[6], vals[7]))
}

impl CpuState {
    /// Feed the raw contents of `/proc/stat` and get back the CPU % (0–100).
    pub fn update(&mut self, proc_stat: &str) -> f32 {
        let first_line = proc_stat.lines().next().unwrap_or("");
        let Some((user, nice, system, idle, iowait, irq, softirq, steal)) =
            parse_cpu_stat(first_line)
        else {
            return 0.0;
        };

        let active = user + nice + system + irq + softirq + steal;
        let total = active + idle + iowait;

        let delta_active = active.saturating_sub(self.prev_active);
        let delta_total = total.saturating_sub(self.prev_total);

        let result = if !self.initialized || delta_total == 0 {
            0.0
        } else {
            (100 * delta_active / delta_total) as f32
        };

        self.prev_active = active;
        self.prev_total = total;
        self.initialized = true;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_STAT_1: &str =
        "cpu  100 20 50 800 10 5 3 2 0 0\ncpu0 50 10 25 400 5 2 1 1 0 0\n";
    const SAMPLE_STAT_2: &str =
        "cpu  200 20 100 900 10 10 3 2 0 0\ncpu0 100 10 50 450 5 5 1 1 0 0\n";

    #[test]
    fn parse_cpu_stat_extracts_fields() {
        let (user, nice, system, idle, iowait, irq, softirq, steal) =
            parse_cpu_stat("cpu  100 20 50 800 10 5 3 2").unwrap();
        assert_eq!((user, nice, system, idle, iowait, irq, softirq, steal),
                   (100, 20, 50, 800, 10, 5, 3, 2));
    }

    #[test]
    fn parse_cpu_stat_returns_none_for_non_cpu_line() {
        assert!(parse_cpu_stat("mem  100 200").is_none());
    }

    #[test]
    fn first_update_returns_zero() {
        let mut state = CpuState::default();
        // First call has no previous sample → 0 %
        let pct = state.update(SAMPLE_STAT_1);
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn second_update_computes_delta() {
        let mut state = CpuState::default();
        state.update(SAMPLE_STAT_1);
        // Between sample 1 and 2:
        //   active delta = (200+20+100+10+3+2) - (100+20+50+5+3+2) = 335 - 180 = 155
        //   total  delta = 335+900+10 - 180+800+10 = 1245 - 990 = 255 → wait no, let me recalculate
        // sample1: active = 100+20+50+5+3+2=180, total = 180+800+10=990
        // sample2: active = 200+20+100+10+3+2=335, total = 335+900+10=1245
        // delta_active = 155, delta_total = 255
        // pct = 100*155/255 = 60 (integer div)
        let pct = state.update(SAMPLE_STAT_2);
        assert_eq!(pct, 60.0);
    }

    #[test]
    fn no_change_returns_zero_percent() {
        let mut state = CpuState::default();
        state.update(SAMPLE_STAT_1);
        let pct = state.update(SAMPLE_STAT_1); // identical → delta = 0
        assert_eq!(pct, 0.0);
    }
}
