/// RAM use metric
use super::MetricMetadata;
use crate::metric::{Metric, MetricEntry};
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt};

/// Memory information
pub struct MemoryUse {
    last_refresh: Instant,
    refresh_frequency: Duration,
    sys: System,
    ram_bytes_total: f32,
    ram_bytes_used: f32,
    swap_bytes_total: f32,
    swap_bytes_used: f32,
}

impl MemoryUse {
    /// Creates a new memory metric
    pub fn new() -> Self {
        let mut metric = Self {
            last_refresh: Instant::now(),
            refresh_frequency: Duration::from_millis(200),
            sys: System::new(),
            ram_bytes_total: 0.,
            ram_bytes_used: 0.,
            swap_bytes_total: 0.,
            swap_bytes_used: 0.,
        };
        metric.refresh();
        metric
    }

    fn refresh(&mut self) {
        self.sys.refresh_memory();
        self.last_refresh = Instant::now();

        // bytes of RAM available
        self.ram_bytes_total = self.sys.total_memory() as f32;

        // bytes of RAM in use
        self.ram_bytes_used = self.sys.used_memory() as f32;

        // bytes of swap available
        self.swap_bytes_total = self.sys.total_swap() as f32;

        // bytes of swap in use
        self.swap_bytes_total = self.sys.used_swap() as f32;
    }
}

impl Default for MemoryUse {
    fn default() -> Self {
        MemoryUse::new()
    }
}

impl Metric for MemoryUse {
    const NAME: &'static str = "CPU Memory";

    type Input = ();

    fn update(&mut self, _item: &Self::Input, _metadata: &MetricMetadata) -> MetricEntry {
        if self.last_refresh.elapsed() >= self.refresh_frequency {
            self.refresh();
        }

        let ram_use_percentage = (self.ram_bytes_used / self.ram_bytes_total) * 100.;
        let swap_use_percentage = (self.swap_bytes_used / self.swap_bytes_total) * 100.;

        let formatted = format!(
            "RAM Used: {:.2}% - Swap Used: {:.2}%",
            ram_use_percentage, swap_use_percentage
        );

        let raw = format!(
            "ram: {:.2}%, swap: {:.2}%",
            ram_use_percentage, swap_use_percentage
        );

        MetricEntry::new(Self::NAME.to_string(), formatted, raw)
    }

    fn clear(&mut self) {}
}
