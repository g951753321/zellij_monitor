use std::collections::BTreeMap;
use std::io::Write;
use zellij_tile::prelude::*;

mod config;
mod metrics;
mod render;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn _start() {
    extern "C" {
        fn __wasm_call_ctors();
    }
    unsafe { __wasm_call_ctors() };
}

use config::{Config, MetricType};
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
    // CPU temperature
    cpu_temp_celsius: f32,
    // Disk (populated from run_command result)
    disk_used_pct: u8,
    disk_avail_mib: u64,
    // Elapsed seconds between timer ticks, used for network rate calculation
    // when RunCommandResult arrives asynchronously.
    last_elapsed_s: f64,
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
        ]);
        subscribe(&[
            EventType::Timer,
            EventType::RunCommandResult,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                eprintln!("[zellij_monitor] permissions granted");
                self.permissions_granted = true;
                set_selectable(false);
                self.request_all_metrics(0.0);
                set_timeout(self.config.refresh_interval as f64);
                eprintln!("[zellij_monitor] initial metrics requested, timer armed");
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                eprintln!("[zellij_monitor] permissions denied");
                self.permissions_granted = false;
                true
            }
            Event::Timer(elapsed) => {
                eprintln!("[zellij_monitor] timer tick elapsed={:.2}s", elapsed);
                if self.permissions_granted {
                    self.request_all_metrics(elapsed);
                }
                set_timeout(self.config.refresh_interval as f64);
                true
            }
            Event::RunCommandResult(exit_code, stdout, _stderr, context) => {
                if exit_code != Some(0) {
                    return true;
                }
                let text = String::from_utf8_lossy(&stdout);
                match context.get("metric").map(|s| s.as_str()) {
                    Some("cpu") => {
                        self.cpu_pct = self.cpu.update(&text);
                    }
                    Some("memory") => {
                        let (used, total) = metrics::memory::parse_meminfo(&text);
                        self.mem_used_mib = used;
                        self.mem_total_mib = total;
                    }
                    Some("loadavg") => {
                        let (l1, l5, l15) = metrics::loadavg::parse_loadavg(&text);
                        self.load_1 = l1;
                        self.load_5 = l5;
                        self.load_15 = l15;
                    }
                    Some("network") => {
                        let (rx, tx) = self.net.update(
                            &text,
                            &self.config.network_interface,
                            self.last_elapsed_s,
                        );
                        self.net_rx_kbps = rx;
                        self.net_tx_kbps = tx;
                    }
                    Some("cpu_temp") => {
                        self.cpu_temp_celsius =
                            metrics::cpu_temp::parse_thermal_zones(&text);
                    }
                    Some("disk") => {
                        let (pct, avail) = metrics::disk::parse_df_output(&text);
                        self.disk_used_pct = pct;
                        self.disk_avail_mib = avail;
                    }
                    _ => {}
                }
                eprintln!(
                    "[zellij_monitor] metric={:?} cpu={:.1}% mem={}/{} MiB load={:.2}",
                    context.get("metric"),
                    self.cpu_pct,
                    self.mem_used_mib,
                    self.mem_total_mib,
                    self.load_1,
                );
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let output = render::render_bar(self, cols);
        eprintln!(
            "[zellij_monitor] render cols={} output_len={}",
            cols,
            output.len()
        );
        print!("{}", output);
        let _ = std::io::stdout().flush();
    }
}

impl State {
    fn request_all_metrics(&mut self, elapsed_s: f64) {
        self.last_elapsed_s = elapsed_s;
        eprintln!(
            "[zellij_monitor] requesting metrics elapsed_s={:.2}",
            elapsed_s
        );

        for mt in &self.config.plugins {
            let mut ctx = BTreeMap::new();
            match mt {
                MetricType::Cpu => {
                    ctx.insert("metric".into(), "cpu".into());
                    run_command(&["cat", "/proc/stat"], ctx);
                }
                MetricType::Memory => {
                    ctx.insert("metric".into(), "memory".into());
                    run_command(&["cat", "/proc/meminfo"], ctx);
                }
                MetricType::LoadAvg => {
                    ctx.insert("metric".into(), "loadavg".into());
                    run_command(&["cat", "/proc/loadavg"], ctx);
                }
                MetricType::Network => {
                    ctx.insert("metric".into(), "network".into());
                    run_command(&["cat", "/proc/net/dev"], ctx);
                }
                MetricType::CpuTemp => {
                    ctx.insert("metric".into(), "cpu_temp".into());
                    run_command(
                        &["sh", "-c", "cat /sys/class/thermal/thermal_zone*/temp"],
                        ctx,
                    );
                }
                MetricType::Disk => {
                    ctx.insert("metric".into(), "disk".into());
                    run_command(&["df", "-BM", self.config.disk_path.as_str()], ctx);
                }
            }
        }
    }
}
