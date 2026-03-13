use crate::config::{Alignment, MetricType};
use crate::State;

// ANSI 24-bit colour helpers
fn fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

fn bg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{r};{g};{b}m")
}

const RESET: &str = "\x1b[0m";

// Palette
const BG: (u8, u8, u8) = (30, 30, 36);        // dark background
const FG_LABEL: (u8, u8, u8) = (150, 150, 170); // dim label
const FG_OK: (u8, u8, u8) = (100, 220, 120);    // green — below warn
const FG_WARN: (u8, u8, u8) = (255, 200, 50);   // yellow — 10 % below warn
const FG_CRIT: (u8, u8, u8) = (240, 70, 70);    // red — at or above warn

const SEP: &str = "  │  ";

/// Pick a value colour given the current value and the warning threshold.
fn value_color(value_pct: u8, warn_pct: u8) -> String {
    if value_pct >= warn_pct {
        fg(FG_CRIT.0, FG_CRIT.1, FG_CRIT.2)
    } else if value_pct + 10 >= warn_pct {
        fg(FG_WARN.0, FG_WARN.1, FG_WARN.2)
    } else {
        fg(FG_OK.0, FG_OK.1, FG_OK.2)
    }
}

/// Format bytes/s as a human-readable rate string.
fn fmt_rate(kbps: f64) -> String {
    if kbps >= 1024.0 * 1024.0 {
        format!("{:.1}GB/s", kbps / (1024.0 * 1024.0))
    } else if kbps >= 1024.0 {
        format!("{:.1}MB/s", kbps / 1024.0)
    } else {
        format!("{:.0}KB/s", kbps)
    }
}

/// Render the full status bar string (with ANSI codes) truncated to `cols`.
pub fn render_bar(state: &State, cols: usize) -> String {
    let mut segments: Vec<String> = Vec::new();

    let label = |s: &str| -> String {
        format!("{}{}{}", fg(FG_LABEL.0, FG_LABEL.1, FG_LABEL.2), s, RESET)
    };

    for mt in &state.config.plugins {
        match mt {
            MetricType::Cpu => {
                let pct = state.cpu_pct as u8;
                let col = value_color(pct, state.config.cpu_warn_pct);
                segments.push(format!(
                    "{}CPU {}{:3}%{}",
                    label(""),
                    col,
                    pct,
                    RESET
                ));
            }
            MetricType::Memory if state.mem_total_mib > 0 => {
                let used = state.mem_used_mib;
                let total = state.mem_total_mib;
                let pct = (used * 100 / total.max(1)) as u8;
                let col = value_color(pct, state.config.mem_warn_pct);

                let (used_disp, total_disp, unit) = if total >= 1024 {
                    (used as f64 / 1024.0, total as f64 / 1024.0, "GiB")
                } else {
                    (used as f64, total as f64, "MiB")
                };
                segments.push(format!(
                    "{}MEM {}{:.1}/{:.1} {}{}",
                    label(""),
                    col,
                    used_disp,
                    total_disp,
                    unit,
                    RESET
                ));
            }
            MetricType::CpuTemp => {
                let temp = state.cpu_temp_celsius as u8;
                let col = value_color(temp, state.config.cpu_temp_warn);
                segments.push(format!(
                    "{}TEMP {}{:3}°C{}",
                    label(""),
                    col,
                    temp,
                    RESET
                ));
            }
            MetricType::Disk => {
                let pct = state.disk_used_pct;
                let col = value_color(pct, state.config.disk_warn_pct);
                segments.push(format!(
                    "{}DISK {}{:3}% {} free{}",
                    label(""),
                    col,
                    pct,
                    fmt_mib(state.disk_avail_mib),
                    RESET
                ));
            }
            MetricType::Network => {
                let rx_col = fg(FG_OK.0, FG_OK.1, FG_OK.2);
                let tx_col = fg(FG_WARN.0, FG_WARN.1, FG_WARN.2);
                segments.push(format!(
                    "{}NET {}↓{} {}↑{}{}",
                    label(""),
                    rx_col,
                    fmt_rate(state.net_rx_kbps),
                    tx_col,
                    fmt_rate(state.net_tx_kbps),
                    RESET
                ));
            }
            MetricType::LoadAvg => {
                segments.push(format!(
                    "{}LOAD {}{:.2} {:.2} {:.2}{}",
                    label(""),
                    fg(FG_OK.0, FG_OK.1, FG_OK.2),
                    state.load_1,
                    state.load_5,
                    state.load_15,
                    RESET
                ));
            }
            _ => {} // Memory with total == 0 — skip
        }
    }

    if !state.permissions_granted && !state.initialized {
        let msg = " ⏳ Waiting for permissions…";
        return format!(
            "{}{}{}{}",
            bg(BG.0, BG.1, BG.2),
            fg(FG_LABEL.0, FG_LABEL.1, FG_LABEL.2),
            truncate_visual(msg, cols),
            RESET
        );
    }

    // Join segments with a separator
    let sep = format!(
        "{}{}{}",
        fg(FG_LABEL.0, FG_LABEL.1, FG_LABEL.2),
        SEP,
        RESET
    );
    let content = format!("{} ", segments.join(&sep));

    // We can't easily measure ANSI-escaped length, so we build a plain-text
    // version to know if truncation is needed, then ship the coloured one.
    let plain = strip_ansi(&content);
    let plain_len = plain.len();

    if plain_len > cols {
        let mut segs = segments.clone();
        loop {
            if segs.is_empty() {
                return String::new();
            }
            let candidate = format!(" {} ", segs.join(&sep));
            let cand_len = strip_ansi(&candidate).len();
            if cand_len <= cols {
                return aligned(cols, cand_len, &candidate, state.config.alignment);
            }
            segs.pop();
        }
    } else {
        aligned(cols, plain_len, &content, state.config.alignment)
    }
}

/// Wrap `content` with background colour and pad according to `alignment`.
fn aligned(cols: usize, content_len: usize, content: &str, alignment: Alignment) -> String {
    let pad = cols.saturating_sub(content_len);
    let bg_str = bg(BG.0, BG.1, BG.2);

    match alignment {
        Alignment::Left => {
            format!(
                "{}{}{}{:>width$}{}",
                bg_str, content, RESET, "", RESET,
                width = pad
            )
        }
        Alignment::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!(
                "{:>lw$}{}{}{}{:>rw$}{}",
                "", bg_str, content, RESET, "", RESET,
                lw = left, rw = right
            )
        }
        Alignment::Right => {
            format!(
                "{:>width$}{}{}{}",
                "", bg_str, content, RESET,
                width = pad
            )
        }
    }
}

fn fmt_mib(mib: u64) -> String {
    if mib >= 1024 {
        format!("{:.1}GiB", mib as f64 / 1024.0)
    } else {
        format!("{mib}MiB")
    }
}

/// Truncate a plain string to `max_cols` characters.
fn truncate_visual(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    s.chars().take(max_cols).collect()
}

/// Remove ANSI escape sequences so we can measure visual width.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_escape_codes() {
        let s = "\x1b[38;2;100;220;120mOK\x1b[0m";
        assert_eq!(strip_ansi(s), "OK");
    }

    #[test]
    fn truncate_visual_limits_length() {
        let s = "Hello World";
        assert_eq!(truncate_visual(s, 5), "Hello");
        assert_eq!(truncate_visual(s, 20), "Hello World");
        assert_eq!(truncate_visual(s, 0), "");
    }

    #[test]
    fn value_color_ok_below_warn() {
        let col = value_color(50, 80);
        assert_eq!(col, fg(FG_OK.0, FG_OK.1, FG_OK.2));
    }

    #[test]
    fn value_color_warn_near_threshold() {
        let col = value_color(72, 80); // within 10 %
        assert_eq!(col, fg(FG_WARN.0, FG_WARN.1, FG_WARN.2));
    }

    #[test]
    fn value_color_crit_at_threshold() {
        let col = value_color(80, 80);
        assert_eq!(col, fg(FG_CRIT.0, FG_CRIT.1, FG_CRIT.2));
    }

    #[test]
    fn fmt_rate_scales_units() {
        assert!(fmt_rate(500.0).ends_with("KB/s"));
        assert!(fmt_rate(2048.0).ends_with("MB/s"));
        assert!(fmt_rate(2.0 * 1024.0 * 1024.0).ends_with("GB/s"));
    }

    #[test]
    fn render_bar_respects_narrow_terminal() {
        let mut state = State::default();
        state.permissions_granted = true;
        state.initialized = true;
        state.cpu_pct = 42.0;
        state.mem_used_mib = 4000;
        state.mem_total_mib = 16000;
        state.load_1 = 0.5;
        state.load_5 = 0.6;
        state.load_15 = 0.7;

        let output = render_bar(&state, 20);
        // Visual width of the stripped output must be ≤ 20
        assert!(strip_ansi(&output).len() <= 20);
    }
}
