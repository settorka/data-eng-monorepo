use std::sync::atomic::{AtomicU64, Ordering};

/// Global counters for correctness + operations.
/// These exist to prove system invariants, not just count events.
pub struct Metrics {
    /// Number of events acknowledged (202).
    /// Used to detect silent loss vs persisted + DLQ totals.
    pub events_accepted: AtomicU64,

    /// Number of rejected requests (422 / 429 / 503).
    /// Indicates overload or invalid input pressure.
    pub events_rejected: AtomicU64,

    /// Publish failures to broker.
    /// Indicates upstream dependency instability.
    pub publish_failures: AtomicU64,

    /// Messages successfully read from Kafka.
    /// Used to compare against ingestion volume.
    pub events_consumed: AtomicU64,

    /// Events successfully written to Scylla.
    /// Core durability signal.
    pub events_persisted: AtomicU64,

    /// Failed persistence attempts (before DLQ).
    /// Indicates storage instability or schema issues.
    pub persist_failures: AtomicU64,

    /// Events routed to DLQ.
    /// Must remain bounded; spike indicates systemic failure.
    pub dlq_writes: AtomicU64,
}

impl Metrics {
    pub const fn new() -> Self {
        Self {
            events_accepted: AtomicU64::new(0),
            events_rejected: AtomicU64::new(0),
            publish_failures: AtomicU64::new(0),
            events_consumed: AtomicU64::new(0),
            events_persisted: AtomicU64::new(0),
            persist_failures: AtomicU64::new(0),
            dlq_writes: AtomicU64::new(0),
        }
    }
}

/// Single global instance.
/// Atomic-only → no locks, safe in async hot paths.
pub static METRICS: Metrics = Metrics::new();

/// Init hook for future exporters (Prometheus, OTEL).
pub fn init() {}

#[inline]
pub fn inc_accepted() {
    METRICS.events_accepted.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn inc_rejected() {
    METRICS.events_rejected.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn inc_publish_failure() {
    METRICS.publish_failures.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn inc_consumed() {
    METRICS.events_consumed.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn inc_persisted(n: u64) {
    METRICS.events_persisted.fetch_add(n, Ordering::Relaxed);
}

#[inline]
pub fn inc_persist_failure() {
    METRICS.persist_failures.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn inc_dlq() {
    METRICS.dlq_writes.fetch_add(1, Ordering::Relaxed);
}