use std::collections::BTreeMap;

/// Plugin configuration, loaded from the KDL layout file.
///
/// Example KDL configuration:
/// ```kdl
/// plugin location="file:~/.config/zellij/plugins/zellij_monitor.wasm" {
///     show_cpu        "true"
///     show_memory     "true"
///     show_disk       "true"
///     show_network    "true"
///     show_loadavg    "true"
///     refresh_interval "5"
///     disk_path       "/"
///     network_interface "all"
///     cpu_warn_pct    "80"
///     mem_warn_pct    "80"
///     disk_warn_pct   "80"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    pub show_cpu: bool,
    pub show_memory: bool,
    pub show_disk: bool,
    pub show_network: bool,
    pub show_loadavg: bool,
    pub refresh_interval: u64,
    pub disk_path: String,
    pub network_interface: String,
    pub cpu_warn_pct: u8,
    pub mem_warn_pct: u8,
    pub disk_warn_pct: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_cpu: true,
            show_memory: true,
            show_disk: true,
            show_network: true,
            show_loadavg: true,
            refresh_interval: 5,
            disk_path: "/".to_owned(),
            network_interface: "all".to_owned(),
            cpu_warn_pct: 80,
            mem_warn_pct: 80,
            disk_warn_pct: 80,
        }
    }
}

impl Config {
    pub fn from_map(map: &BTreeMap<String, String>) -> Self {
        let mut cfg = Self::default();

        if let Some(v) = map.get("show_cpu") {
            cfg.show_cpu = v != "false";
        }
        if let Some(v) = map.get("show_memory") {
            cfg.show_memory = v != "false";
        }
        if let Some(v) = map.get("show_disk") {
            cfg.show_disk = v != "false";
        }
        if let Some(v) = map.get("show_network") {
            cfg.show_network = v != "false";
        }
        if let Some(v) = map.get("show_loadavg") {
            cfg.show_loadavg = v != "false";
        }
        if let Some(v) = map.get("refresh_interval") {
            cfg.refresh_interval = v.parse::<u64>().unwrap_or(5).max(1);
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
        assert!(cfg.show_cpu);
        assert!(cfg.show_memory);
        assert!(cfg.show_disk);
        assert!(cfg.show_network);
        assert!(cfg.show_loadavg);
        assert_eq!(cfg.refresh_interval, 5);
        assert_eq!(cfg.disk_path, "/");
        assert_eq!(cfg.network_interface, "all");
        assert_eq!(cfg.cpu_warn_pct, 80);
        assert_eq!(cfg.mem_warn_pct, 80);
        assert_eq!(cfg.disk_warn_pct, 80);
    }

    #[test]
    fn overrides_are_applied() {
        let cfg = Config::from_map(&map(&[
            ("show_cpu", "false"),
            ("show_disk", "false"),
            ("refresh_interval", "10"),
            ("disk_path", "/home"),
            ("network_interface", "eth0"),
            ("cpu_warn_pct", "90"),
        ]));
        assert!(!cfg.show_cpu);
        assert!(!cfg.show_disk);
        assert!(cfg.show_memory); // untouched → default true
        assert_eq!(cfg.refresh_interval, 10);
        assert_eq!(cfg.disk_path, "/home");
        assert_eq!(cfg.network_interface, "eth0");
        assert_eq!(cfg.cpu_warn_pct, 90);
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
}
