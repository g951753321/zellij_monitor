use std::collections::BTreeMap;
use zellij_tile::prelude::*;

mod config;
mod metrics;
mod render;

use config::Config;
use metrics::{cpu::CpuState, network::NetworkState};

#[derive(Default)]
struct State {
    config: Config,
    // CPU
    cpu: CpuState,
    cpu_pct: f32,
    // Memory
    mem_used_mib: u64,
    mem_total_mib: u64,
    // Load average
    load_1: f32,
    load_5: f32,
    load_15: f32,
    // Network
    net: NetworkState,
    net_rx_kbps: f64,
    net_tx_kbps: f64,
    // Disk (populated from run_command result)
    disk_used_pct: u8,
    disk_avail_mib: u64,
    // State flags
    initialized: bool,
    permissions_granted: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from_map(&configuration);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::RunCommands,
            PermissionType::FullHdAccess,
        ]);
        subscribe(&[
            EventType::Timer,
            EventType::RunCommandResult,
            EventType::PermissionRequestResult,
        ]);
        set_selectable(false);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.permissions_granted = true;
                // Kick off first collection immediately
                self.collect_proc_metrics();
                self.request_disk_metrics();
                set_timeout(self.config.refresh_interval as f64);
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.permissions_granted = false;
                true
            }
            Event::Timer(_) => {
                if self.permissions_granted {
                    self.collect_proc_metrics();
                    self.request_disk_metrics();
                }
                set_timeout(self.config.refresh_interval as f64);
                true
            }
            Event::RunCommandResult(exit_code, stdout, _stderr, context) => {
                if context.get("metric").map(|s| s == "disk").unwrap_or(false) {
                    if exit_code == Some(0) {
                        let text = String::from_utf8_lossy(&stdout);
                        let (pct, avail) = metrics::disk::parse_df_output(&text);
                        self.disk_used_pct = pct;
                        self.disk_avail_mib = avail;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let output = render::render_bar(self, cols);
        print!("{}", output);
    }
}

impl State {
    fn collect_proc_metrics(&mut self) {
        if self.config.show_cpu {
            if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
                self.cpu_pct = self.cpu.update(&stat);
            }
        }
        if self.config.show_memory {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                let (used, total) = metrics::memory::parse_meminfo(&meminfo);
                self.mem_used_mib = used;
                self.mem_total_mib = total;
            }
        }
        if self.config.show_loadavg {
            if let Ok(la) = std::fs::read_to_string("/proc/loadavg") {
                let (l1, l5, l15) = metrics::loadavg::parse_loadavg(&la);
                self.load_1 = l1;
                self.load_5 = l5;
                self.load_15 = l15;
            }
        }
        if self.config.show_network {
            if let Ok(netdev) = std::fs::read_to_string("/proc/net/dev") {
                let (rx, tx) = self.net.update(&netdev, &self.config.network_interface);
                self.net_rx_kbps = rx;
                self.net_tx_kbps = tx;
            }
        }
    }

    fn request_disk_metrics(&self) {
        if self.config.show_disk {
            let mut ctx = BTreeMap::new();
            ctx.insert("metric".to_owned(), "disk".to_owned());
            run_command(
                &["df", "-BM", self.config.disk_path.as_str()],
                ctx,
            );
        }
    }
}
