//! Performance metrics collection and reporting for ternary systems.
//!
//! Provides `MetricsCollector`, `MetricsRegistry`, `MetricsReport`, and time-series
//! metrics for tracking latency, throughput, and accuracy.

use core::fmt;

/// A single metric sample with a timestamp.
#[derive(Debug, Clone)]
pub struct Sample {
    pub timestamp_ms: u64,
    pub value: f64,
}

/// Time-series of samples for a single metric.
#[derive(Debug, Clone)]
pub struct TimeSeries {
    pub name: String,
    pub samples: Vec<Sample>,
}

impl TimeSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            samples: Vec::new(),
        }
    }

    pub fn push(&mut self, timestamp_ms: u64, value: f64) {
        self.samples.push(Sample {
            timestamp_ms,
            value,
        });
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn mean(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let sum: f64 = self.samples.iter().map(|s| s.value).sum();
        Some(sum / self.samples.len() as f64)
    }

    pub fn min(&self) -> Option<f64> {
        self.samples.iter().map(|s| s.value).reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.samples.iter().map(|s| s.value).reduce(f64::max)
    }

    pub fn latest(&self) -> Option<&Sample> {
        self.samples.last()
    }
}

/// Kinds of metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Latency,
    Throughput,
    Accuracy,
    Custom,
}

/// Collects metric samples for a specific metric kind.
pub struct MetricsCollector {
    pub name: String,
    pub kind: MetricKind,
    pub series: TimeSeries,
}

impl MetricsCollector {
    pub fn new(name: &str, kind: MetricKind) -> Self {
        Self {
            name: name.to_string(),
            kind,
            series: TimeSeries::new(name),
        }
    }

    pub fn record(&mut self, timestamp_ms: u64, value: f64) {
        self.series.push(timestamp_ms, value);
    }

    pub fn sample_count(&self) -> usize {
        self.series.len()
    }

    pub fn average(&self) -> Option<f64> {
        self.series.mean()
    }

    pub fn min_value(&self) -> Option<f64> {
        self.series.min()
    }

    pub fn max_value(&self) -> Option<f64> {
        self.series.max()
    }
}

/// Registry of named metric collectors.
pub struct MetricsRegistry {
    collectors: Vec<MetricsCollector>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            collectors: Vec::new(),
        }
    }

    pub fn register(&mut self, name: &str, kind: MetricKind) -> usize {
        let idx = self.collectors.len();
        self.collectors.push(MetricsCollector::new(name, kind));
        idx
    }

    pub fn record(&mut self, idx: usize, timestamp_ms: u64, value: f64) -> bool {
        if let Some(c) = self.collectors.get_mut(idx) {
            c.record(timestamp_ms, value);
            true
        } else {
            false
        }
    }

    pub fn get(&self, idx: usize) -> Option<&MetricsCollector> {
        self.collectors.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut MetricsCollector> {
        self.collectors.get_mut(idx)
    }

    pub fn len(&self) -> usize {
        self.collectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.collectors.is_empty()
    }

    /// Find collector by name.
    pub fn find_by_name(&self, name: &str) -> Option<usize> {
        self.collectors.iter().position(|c| c.name == name)
    }

    /// Total samples across all collectors.
    pub fn total_samples(&self) -> usize {
        self.collectors.iter().map(|c| c.sample_count()).sum()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A formatted metrics report.
#[derive(Debug, Clone)]
pub struct MetricsReport {
    pub title: String,
    pub entries: Vec<ReportEntry>,
}

#[derive(Debug, Clone)]
pub struct ReportEntry {
    pub metric_name: String,
    pub kind: MetricKind,
    pub count: usize,
    pub mean: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl MetricsReport {
    pub fn from_registry(title: &str, registry: &MetricsRegistry) -> Self {
        let entries = registry
            .collectors
            .iter()
            .map(|c| ReportEntry {
                metric_name: c.name.clone(),
                kind: c.kind,
                count: c.sample_count(),
                mean: c.average(),
                min: c.min_value(),
                max: c.max_value(),
            })
            .collect();
        Self {
            title: title.to_string(),
            entries,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for MetricsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== {} ===", self.title)?;
        for e in &self.entries {
            let mean_str = e.mean.map_or("N/A".to_string(), |v| format!("{:.2}", v));
            let min_str = e.min.map_or("N/A".to_string(), |v| format!("{:.2}", v));
            let max_str = e.max.map_or("N/A".to_string(), |v| format!("{:.2}", v));
            writeln!(
                f,
                "  {} ({:?}): n={}, mean={}, min={}, max={}",
                e.metric_name, e.kind, e.count, mean_str, min_str, max_str
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeseries_push_and_len() {
        let mut ts = TimeSeries::new("latency");
        ts.push(0, 1.0);
        ts.push(100, 2.0);
        assert_eq!(ts.len(), 2);
        assert!(!ts.is_empty());
    }

    #[test]
    fn test_timeseries_mean() {
        let mut ts = TimeSeries::new("test");
        ts.push(0, 10.0);
        ts.push(1, 20.0);
        ts.push(2, 30.0);
        assert_eq!(ts.mean(), Some(20.0));
    }

    #[test]
    fn test_timeseries_empty_mean() {
        let ts = TimeSeries::new("empty");
        assert_eq!(ts.mean(), None);
    }

    #[test]
    fn test_timeseries_min_max() {
        let mut ts = TimeSeries::new("test");
        ts.push(0, 5.0);
        ts.push(1, 15.0);
        ts.push(2, 10.0);
        assert_eq!(ts.min(), Some(5.0));
        assert_eq!(ts.max(), Some(15.0));
    }

    #[test]
    fn test_timeseries_latest() {
        let mut ts = TimeSeries::new("test");
        ts.push(0, 1.0);
        ts.push(100, 2.0);
        let latest = ts.latest().unwrap();
        assert_eq!(latest.value, 2.0);
        assert_eq!(latest.timestamp_ms, 100);
    }

    #[test]
    fn test_collector_record() {
        let mut c = MetricsCollector::new("lat", MetricKind::Latency);
        c.record(0, 1.5);
        c.record(100, 2.5);
        assert_eq!(c.sample_count(), 2);
    }

    #[test]
    fn test_collector_stats() {
        let mut c = MetricsCollector::new("tp", MetricKind::Throughput);
        c.record(0, 100.0);
        c.record(1, 200.0);
        assert_eq!(c.average(), Some(150.0));
        assert_eq!(c.min_value(), Some(100.0));
        assert_eq!(c.max_value(), Some(200.0));
    }

    #[test]
    fn test_registry_register_and_record() {
        let mut reg = MetricsRegistry::new();
        let idx = reg.register("latency", MetricKind::Latency);
        assert!(reg.record(idx, 0, 1.0));
        assert!(reg.record(idx, 100, 2.0));
        assert_eq!(reg.get(idx).unwrap().sample_count(), 2);
    }

    #[test]
    fn test_registry_invalid_index() {
        let mut reg = MetricsRegistry::new();
        assert!(!reg.record(99, 0, 1.0));
    }

    #[test]
    fn test_registry_find_by_name() {
        let mut reg = MetricsRegistry::new();
        reg.register("alpha", MetricKind::Latency);
        let idx = reg.register("beta", MetricKind::Throughput);
        assert_eq!(reg.find_by_name("beta"), Some(idx));
        assert_eq!(reg.find_by_name("gamma"), None);
    }

    #[test]
    fn test_registry_total_samples() {
        let mut reg = MetricsRegistry::new();
        let a = reg.register("a", MetricKind::Accuracy);
        let b = reg.register("b", MetricKind::Latency);
        reg.record(a, 0, 1.0);
        reg.record(a, 1, 2.0);
        reg.record(b, 0, 3.0);
        assert_eq!(reg.total_samples(), 3);
    }

    #[test]
    fn test_report_from_registry() {
        let mut reg = MetricsRegistry::new();
        let idx = reg.register("lat", MetricKind::Latency);
        reg.record(idx, 0, 10.0);
        reg.record(idx, 1, 20.0);
        let report = MetricsReport::from_registry("Test Report", &reg);
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].mean, Some(15.0));
    }

    #[test]
    fn test_report_display() {
        let mut reg = MetricsRegistry::new();
        let idx = reg.register("lat", MetricKind::Latency);
        reg.record(idx, 0, 10.0);
        let report = MetricsReport::from_registry("Demo", &reg);
        let s = format!("{}", report);
        assert!(s.contains("Demo"));
        assert!(s.contains("lat"));
    }

    #[test]
    fn test_report_empty() {
        let reg = MetricsRegistry::new();
        let report = MetricsReport::from_registry("Empty", &reg);
        assert!(report.is_empty());
    }

    #[test]
    fn test_default_registry() {
        let reg = MetricsRegistry::default();
        assert!(reg.is_empty());
    }

    #[test]
    fn test_multiple_collectors_independent() {
        let mut reg = MetricsRegistry::new();
        let a = reg.register("a", MetricKind::Accuracy);
        let b = reg.register("b", MetricKind::Throughput);
        reg.record(a, 0, 0.9);
        reg.record(b, 0, 500.0);
        assert_eq!(reg.get(a).unwrap().average(), Some(0.9));
        assert_eq!(reg.get(b).unwrap().average(), Some(500.0));
    }

    #[test]
    fn test_timeseries_many_samples() {
        let mut ts = TimeSeries::new("bulk");
        for i in 0..1000 {
            ts.push(i, i as f64);
        }
        assert_eq!(ts.len(), 1000);
        assert_eq!(ts.mean(), Some(499.5));
    }

    #[test]
    fn test_collector_empty_stats() {
        let c = MetricsCollector::new("empty", MetricKind::Custom);
        assert_eq!(c.average(), None);
        assert_eq!(c.min_value(), None);
        assert_eq!(c.max_value(), None);
    }

    #[test]
    fn test_timeseries_latest_empty() {
        let ts = TimeSeries::new("empty");
        assert!(ts.latest().is_none());
    }

    #[test]
    fn test_registry_len_and_empty() {
        let reg = MetricsRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_registry_get_mut() {
        let mut reg = MetricsRegistry::new();
        let idx = reg.register("lat", MetricKind::Latency);
        reg.record(idx, 0, 1.0);
        if let Some(c) = reg.get_mut(idx) {
            c.record(1, 99.0);
        }
        assert_eq!(reg.get(idx).unwrap().sample_count(), 2);
    }

    #[test]
    fn test_registry_get_mut_invalid_index() {
        let mut reg = MetricsRegistry::new();
        reg.register("lat", MetricKind::Latency);
        assert!(reg.get_mut(99).is_none());
        assert!(reg.get(99).is_none());
    }

    #[test]
    fn test_report_display_na_for_empty_collector() {
        let mut reg = MetricsRegistry::new();
        let _ = reg.register("idle", MetricKind::Latency);
        let report = MetricsReport::from_registry("Has Gaps", &reg);
        let s = format!("{}", report);
        assert!(s.contains("n=0"), "expected zero sample count, got: {}", s);
        assert!(
            s.contains("mean=N/A") && s.contains("min=N/A") && s.contains("max=N/A"),
            "expected N/A placeholders for empty series, got: {}",
            s,
        );
    }

    #[test]
    fn test_report_single_sample_min_eq_max() {
        let mut reg = MetricsRegistry::new();
        let idx = reg.register("one", MetricKind::Throughput);
        reg.record(idx, 5, 42.0);
        let report = MetricsReport::from_registry("Single", &reg);
        let e = &report.entries[0];
        assert_eq!(e.count, 1);
        assert_eq!(e.mean, Some(42.0));
        assert_eq!(e.min, Some(42.0));
        assert_eq!(e.max, Some(42.0));
        let s = format!("{}", report);
        assert!(
            s.contains("min=42.00") && s.contains("max=42.00"),
            "got: {}",
            s
        );
    }

    #[test]
    fn test_report_clone_is_independent_snapshot() {
        let mut reg = MetricsRegistry::new();
        let idx = reg.register("lat", MetricKind::Latency);
        reg.record(idx, 0, 10.0);
        let report = MetricsReport::from_registry("Orig", &reg);
        let cloned = report.clone();
        assert_eq!(cloned.title, report.title);
        assert_eq!(cloned.entries.len(), report.entries.len());
        assert_eq!(cloned.entries[0].mean, Some(10.0));
        // Report is a snapshot: recording more data must not change either copy.
        reg.record(idx, 1, 999.0);
        let snapshot_again = MetricsReport::from_registry("Orig", &reg);
        assert_eq!(report.entries[0].count, 1);
        assert_eq!(cloned.entries[0].count, 1);
        assert_eq!(snapshot_again.entries[0].count, 2);
    }

    #[test]
    fn test_timeseries_clone_is_independent() {
        let mut ts = TimeSeries::new("lat");
        ts.push(0, 1.0);
        let mirror = ts.clone();
        assert_eq!(mirror.len(), 1);
        ts.push(1, 2.0);
        assert_eq!(ts.len(), 2);
        assert_eq!(mirror.len(), 1, "clone must not see post-clone mutations");
        assert_eq!(mirror.mean(), Some(1.0));
    }

    #[test]
    fn test_timeseries_min_max_empty_direct() {
        let ts = TimeSeries::new("empty");
        assert_eq!(ts.min(), None);
        assert_eq!(ts.max(), None);
        assert_eq!(ts.mean(), None);
    }
}
