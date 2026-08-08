use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// SystemMetrics holds lock-free atomic counters for system telemetry
pub struct MetricsCollector {
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
    active_connections: AtomicU64,
    cpu_usage_milli: AtomicU64, // CPU usage stored as percentage * 100
    memory_used_bytes: AtomicU64,
    route_counters: RwLock<HashMap<String, AtomicU64>>,
    start_time: Instant,
}

impl MetricsCollector {
    /// Instantiates a new thread-safe lock-free MetricsCollector
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            cpu_usage_milli: AtomicU64::new(0),
            memory_used_bytes: AtomicU64::new(0),
            route_counters: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    /// Atomically increments total requests handled
    pub fn increment_request_count(&self, method: &str, path: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        let key = format!("{}:{}", method, path);

        if let Ok(map) = self.route_counters.read() {
            if let Some(counter) = map.get(&key) {
                counter.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        if let Ok(mut map) = self.route_counters.write() {
            map.entry(key)
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Atomically records current CPU usage percentage
    pub fn record_cpu_usage(&self, percentage: f64) {
        let milli = (percentage * 100.0) as u64;
        self.cpu_usage_milli.store(milli, Ordering::Relaxed);
    }

    /// Atomically records current Memory usage in bytes
    pub fn record_memory_bytes(&self, bytes: u64) {
        self.memory_used_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Atomically increments failed requests count
    pub fn record_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Exports all recorded metrics formatted per Prometheus exposition standard
    pub fn export_prometheus_format(&self) -> String {
        let uptime_secs = self.start_time.elapsed().as_secs();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let total_req = self.total_requests.load(Ordering::Relaxed);
        let failed_req = self.failed_requests.load(Ordering::Relaxed);
        let cpu = self.cpu_usage_milli.load(Ordering::Relaxed) as f64 / 100.0;
        let memory = self.memory_used_bytes.load(Ordering::Relaxed);

        let mut output = String::with_capacity(1024);

        output.push_str("# HELP system_uptime_seconds Total runtime of exporter in seconds\n");
        output.push_str("# TYPE system_uptime_seconds counter\n");
        output.push_str(&format!("system_uptime_seconds {}\n\n", uptime_secs));

        output.push_str("# HELP http_requests_total Total number of HTTP requests processed\n");
        output.push_str("# TYPE http_requests_total counter\n");
        output.push_str(&format!("http_requests_total {}\n\n", total_req));

        output.push_str("# HELP http_requests_failed_total Total failed HTTP requests\n");
        output.push_str("# TYPE http_requests_failed_total counter\n");
        output.push_str(&format!("http_requests_failed_total {}\n\n", failed_req));

        output.push_str("# HELP system_cpu_usage_percent Realtime system CPU usage percentage\n");
        output.push_str("# TYPE system_cpu_usage_percent gauge\n");
        output.push_str(&format!("system_cpu_usage_percent {:.2}\n\n", cpu));

        output.push_str("# HELP system_memory_usage_bytes Realtime RSS memory usage in bytes\n");
        output.push_str("# TYPE system_memory_usage_bytes gauge\n");
        output.push_str(&format!("system_memory_usage_bytes {}\n\n", memory));

        if let Ok(map) = self.route_counters.read() {
            for (route, counter) in map.iter() {
                let parts: Vec<&str> = route.splitn(2, ':').collect();
                if parts.len() == 2 {
                    output.push_str(&format!(
                        "http_route_requests_total{{method=\"{}\",path=\"{}\"}} {} {}\n",
                        parts[0], parts[1], counter.load(Ordering::Relaxed), timestamp
                    ));
                }
            }
        }

        output
    }
}
