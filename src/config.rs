use std::collections::BTreeMap;

/// Horizontal alignment of the status bar content.
///
/// Configurable via the `alignment` key in the KDL layout:
/// - `"left"` or `"<"` — left-aligned (default)
/// - `"center"` or `"^"` — centered
/// - `"right"` or `">"` — right-aligned
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

impl Alignment {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "<" | "left" => Some(Self::Left),
            "^" | "center" => Some(Self::Center),
            ">" | "right" => Some(Self::Right),
            _ => None,
        }
    }
}

/// Supported metric types for the status bar.
///
/// Supported values: `cpu`, `memory`, `cpu_temp`, `disk`, `network`, `loadavg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Cpu,
    Memory,
    CpuTemp,
    Disk,
    Network,
    LoadAvg,
}

impl MetricType {
    /// Parse a single metric name (case-insensitive, trimmed).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "cpu" => Some(Self::Cpu),
            "memory" | "mem" => Some(Self::Memory),
            "cpu_temp" | "temp" => Some(Self::CpuTemp),
            "disk" => Some(Self::Disk),
            "network" | "net" => Some(Self::Network),
            "loadavg" | "load" => Some(Self::LoadAvg),
            _ => None,
        }
    }

    /// All metric types in default display order.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Cpu,
            Self::Memory,
            Self::CpuTemp,
            Self::Disk,
            Self::Network,
            Self::LoadAvg,
        ]
    }
}

/// Plugin configuration, loaded from the KDL layout file.
///
/// Example KDL configuration:
/// ```kdl
/// plugin location="file:~/.config/zellij/plugins/zellij_monitor.wasm" {
///     plugins          "cpu, memory, cpu_temp"
///     refresh_interval "5"
///     alignment        "left"
///     disk_path        "/"
///     network_interface "all"
///     cpu_warn_pct     "80"
///     mem_warn_pct     "80"
///     disk_warn_pct    "80"
///     cpu_temp_warn    "80"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Ordered list of metrics to display. Only metrics listed here are shown.
    pub plugins: Vec<MetricType>,
    pub refresh_interval: u64,
    pub alignment: Alignment,
    pub disk_path: String,
    pub network_interface: String,
    pub cpu_warn_pct: u8,
    pub mem_warn_pct: u8,
    pub disk_warn_pct: u8,
    /// Warning threshold for CPU temperature in °C.
    pub cpu_temp_warn: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plugins: MetricType::all(),
            refresh_interval: 5,
            alignment: Alignment::default(),
            disk_path: "/".to_owned(),
            network_interface: "all".to_owned(),
            cpu_warn_pct: 80,
            mem_warn_pct: 80,
            disk_warn_pct: 80,
            cpu_temp_warn: 80,
        }
    }
}

impl Config {
    /// Returns `true` if the given metric type is enabled (present in `plugins`).
    #[allow(dead_code)]
    pub fn is_enabled(&self, mt: MetricType) -> bool {
        self.plugins.contains(&mt)
    }

    pub fn from_map(map: &BTreeMap<String, String>) -> Self {
        let mut cfg = Self::default();

        if let Some(v) = map.get("plugins") {
            let parsed: Vec<MetricType> = v
                .split(',')
                .filter_map(|s| MetricType::from_str(s))
                .collect();
            if !parsed.is_empty() {
                cfg.plugins = parsed;
            }
        }
        if let Some(v) = map.get("refresh_interval") {
            cfg.refresh_interval = v.parse::<u64>().unwrap_or(5).max(1);
        }
        if let Some(v) = map.get("alignment") {
            cfg.alignment = Alignment::from_str(v).unwrap_or_default();
        }
        if let Some(v) = map.get("disk_path") {
            cfg.disk_path = v.clone();
        }
        if let Some(v) = map.get("network_interface") {
            cfg.network_interface = v.clone();
        }
        if let Some(v) = map.get("cpu_warn_pct") {
            cfg.cpu_warn_pct = v.parse::<u8>().unwrap_or(80);
        }
        if let Some(v) = map.get("mem_warn_pct") {
            cfg.mem_warn_pct = v.parse::<u8>().unwrap_or(80);
        }
        if let Some(v) = map.get("disk_warn_pct") {
            cfg.disk_warn_pct = v.parse::<u8>().unwrap_or(80);
        }
        if let Some(v) = map.get("cpu_temp_warn") {
            cfg.cpu_temp_warn = v.parse::<u8>().unwrap_or(80);
        }

        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn defaults_when_empty_map() {
        let cfg = Config::from_map(&BTreeMap::new());
        assert_eq!(cfg.plugins, MetricType::all());
        assert_eq!(cfg.refresh_interval, 5);
        assert_eq!(cfg.alignment, Alignment::Left);
        assert_eq!(cfg.disk_path, "/");
        assert_eq!(cfg.network_interface, "all");
        assert_eq!(cfg.cpu_warn_pct, 80);
        assert_eq!(cfg.mem_warn_pct, 80);
        assert_eq!(cfg.disk_warn_pct, 80);
        assert_eq!(cfg.cpu_temp_warn, 80);
    }

    #[test]
    fn order_controls_enabled_metrics() {
        let cfg = Config::from_map(&map(&[
            ("plugins", "cpu, memory, cpu_temp"),
            ("refresh_interval", "10"),
            ("disk_path", "/home"),
            ("network_interface", "eth0"),
            ("cpu_warn_pct", "90"),
        ]));
        assert_eq!(
            cfg.plugins,
            vec![MetricType::Cpu, MetricType::Memory, MetricType::CpuTemp]
        );
        assert!(cfg.is_enabled(MetricType::Cpu));
        assert!(cfg.is_enabled(MetricType::Memory));
        assert!(cfg.is_enabled(MetricType::CpuTemp));
        assert!(!cfg.is_enabled(MetricType::Disk));
        assert!(!cfg.is_enabled(MetricType::Network));
        assert!(!cfg.is_enabled(MetricType::LoadAvg));
        assert_eq!(cfg.refresh_interval, 10);
        assert_eq!(cfg.disk_path, "/home");
        assert_eq!(cfg.network_interface, "eth0");
        assert_eq!(cfg.cpu_warn_pct, 90);
    }

    #[test]
    fn order_accepts_aliases() {
        let cfg = Config::from_map(&map(&[("plugins", "mem, temp, net, load")]));
        assert_eq!(
            cfg.plugins,
            vec![
                MetricType::Memory,
                MetricType::CpuTemp,
                MetricType::Network,
                MetricType::LoadAvg,
            ]
        );
    }

    #[test]
    fn order_ignores_unknown_values() {
        let cfg = Config::from_map(&map(&[("plugins", "cpu, bogus, memory")]));
        assert_eq!(cfg.plugins, vec![MetricType::Cpu, MetricType::Memory]);
    }

    #[test]
    fn order_all_invalid_falls_back_to_default() {
        let cfg = Config::from_map(&map(&[("plugins", "bogus, nope")]));
        assert_eq!(cfg.plugins, MetricType::all());
    }

    #[test]
    fn invalid_values_fall_back_to_defaults() {
        let cfg = Config::from_map(&map(&[
            ("refresh_interval", "abc"),
            ("cpu_warn_pct", "999"), // overflows u8 → parse error → default
        ]));
        assert_eq!(cfg.refresh_interval, 5);
        assert_eq!(cfg.cpu_warn_pct, 80);
    }

    #[test]
    fn refresh_interval_minimum_is_1() {
        let cfg = Config::from_map(&map(&[("refresh_interval", "0")]));
        assert_eq!(cfg.refresh_interval, 1);
    }

    #[test]
    fn alignment_parses_word_forms() {
        assert_eq!(Alignment::from_str("left"), Some(Alignment::Left));
        assert_eq!(Alignment::from_str("center"), Some(Alignment::Center));
        assert_eq!(Alignment::from_str("right"), Some(Alignment::Right));
    }

    #[test]
    fn alignment_parses_symbol_forms() {
        assert_eq!(Alignment::from_str("<"), Some(Alignment::Left));
        assert_eq!(Alignment::from_str("^"), Some(Alignment::Center));
        assert_eq!(Alignment::from_str(">"), Some(Alignment::Right));
    }

    #[test]
    fn alignment_is_case_insensitive() {
        assert_eq!(Alignment::from_str("LEFT"), Some(Alignment::Left));
        assert_eq!(Alignment::from_str("Center"), Some(Alignment::Center));
        assert_eq!(Alignment::from_str("RIGHT"), Some(Alignment::Right));
    }

    #[test]
    fn alignment_invalid_falls_back_to_default() {
        let cfg = Config::from_map(&map(&[("alignment", "bogus")]));
        assert_eq!(cfg.alignment, Alignment::Left);
    }

    #[test]
    fn alignment_from_config_map() {
        let cfg = Config::from_map(&map(&[("alignment", ">")]));
        assert_eq!(cfg.alignment, Alignment::Right);

        let cfg = Config::from_map(&map(&[("alignment", "center")]));
        assert_eq!(cfg.alignment, Alignment::Center);
    }
}
