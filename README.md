# ternary-metrics: Time-series metrics collection and reporting for ternary systems

Record latency, throughput, accuracy, and custom metric samples over time. Aggregate, summarize, and format reports from a registry of named collectors.

## Why This Exists

Ternary strategy systems need performance observability. When you're running thousands of decisions per second, you need to know: Is latency spiking? Is accuracy degrading? Is throughput keeping up? This crate provides a lightweight, allocation-friendly metrics layer that tracks timestamped samples per metric, computes statistics, and generates formatted reports. No external dependencies, no async runtime, no network sinks—just numbers and summaries.

## Core Concepts

- **Sample** — A single timestamped measurement: `(timestamp_ms, value)`. Timestamps are millisecond-epoch `u64`; values are `f64`.
- **TimeSeries** — An ordered list of samples for one metric. Computes `mean`, `min`, `max`, `latest`.
- **MetricKind** — What category of metric: `Latency`, `Throughput`, `Accuracy`, or `Custom`.
- **MetricsCollector** — Wraps a `TimeSeries` with a name and kind. Call `record(timestamp, value)` to append samples.
- **MetricsRegistry** — Owns a `Vec` of collectors. Register by name+kind, record by index, look up by name.
- **MetricsReport** — A snapshot of all collectors with computed statistics (count, mean, min, max per metric). Implements `Display` for formatted output.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-metrics = "0.1"
```

```rust
use ternary_metrics::*;

// Create a registry and register metrics
let mut reg = MetricsRegistry::new();
let lat = reg.register("decision_latency_ms", MetricKind::Latency);
let tp = reg.register("decisions_per_sec", MetricKind::Throughput);
let acc = reg.register("classification_accuracy", MetricKind::Accuracy);

// Record samples (timestamp_ms, value)
reg.record(lat, 1000, 12.5);
reg.record(lat, 2000, 15.0);
reg.record(tp, 1000, 500.0);
reg.record(acc, 1000, 0.92);
reg.record(acc, 2000, 0.95);

// Generate a report
let report = MetricsReport::from_registry("Agent Performance", &reg);
println!("{}", report);
// Output:
// === Agent Performance ===
//   decision_latency_ms (Latency): n=2, mean=13.75, min=12.50, max=15.00
//   decisions_per_sec (Throughput): n=1, mean=500.00, min=500.00, max=500.00
//   classification_accuracy (Accuracy): n=2, mean=0.94, min=0.92, max=0.95
```

## API Overview

| Type / Function | What it is |
|---|---|
| `Sample` | One timestamped measurement |
| `TimeSeries` | Ordered list of samples with statistics |
| `MetricKind` | Enum: `Latency`, `Throughput`, `Accuracy`, `Custom` |
| `MetricsCollector` | Named, typed time-series recorder |
| `MetricsRegistry` | Collection of collectors; index-based access |
| `MetricsReport` | Snapshot with formatted display |
| `ReportEntry` | Per-metric summary: name, kind, count, mean, min, max |

## How It Works

**Registry pattern.** `MetricsRegistry::register` appends a new `MetricsCollector` and returns its index. You store that index and use it for all subsequent `record` calls—O(1) per sample with no hashmap lookup.

**Time series statistics.** `mean()`, `min()`, `max()` are computed by iterating all samples on each call. This is O(n) per query. For high-frequency polling of statistics on large series, consider the trade-off; the design prioritizes simplicity over incremental stat maintenance.

**Report generation.** `MetricsReport::from_registry` iterates all collectors once and captures their current statistics into `ReportEntry` structs. The report is a snapshot—it doesn't update if more samples are recorded after creation.

**Display formatting.** `MetricsReport` implements `std::fmt::Display`, producing a human-readable table with metric name, kind, sample count, and statistics rounded to two decimal places.

## Known Limitations

- **No expiration or downsampling.** Samples accumulate indefinitely. A long-running process with high-frequency recording will use unbounded memory. You'll need to externally manage sample retention.
- **Statistics are O(n) per call.** There's no incremental tracking of mean/min/max. Each call to `average()`, `min_value()`, or `max_value()` iterates the full sample list.
- **No thread safety.** `MetricsRegistry` and `MetricsCollector` are not `Send`/`Sync`. Use within a single task or wrap in a `Mutex` yourself.
- **Percentiles are not available.** Only mean, min, and max. If you need p50/p95/p99, you'd need to sort samples yourself or extend the crate.

## Use Cases

- **Agent performance monitoring.** Record decision latency and accuracy at each timestep. Generate a report at the end of a simulation run to compare strategies.
- **Load testing.** Track throughput and latency under varying load. Use `find_by_name` to locate specific metrics and `total_samples` to confirm all data was collected.
- **A/B comparison.** Create two registries for two strategy variants, record identical workloads, and compare reports side by side.

## Ecosystem Context

This is a utility crate used across the ternary ecosystem. `ternary-pipeline` stages may record metrics. `ternary-replay` can use it to track replay performance. `ternary-scoring` reports can reference metrics data. It has no dependencies on other ternary crates.

## License

MIT
