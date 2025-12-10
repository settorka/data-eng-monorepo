use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use tokio::time;
use tracing::info;

// Global counters
pub static TOTAL_EVENTS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static TOTAL_ERRORS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

/// Spawns a background metrics reporter with configurable interval.
///
/// # Arguments
/// * `interval_secs` - reporting frequency in seconds (default: 5)
///
/// Example:
/// ```rust
/// metrics::logger::init_with_interval(10); // log every 10s
/// ```
pub fn init_with_interval(interval_secs: u64) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            let elapsed = START_TIME.elapsed().as_secs_f64();
            let total = TOTAL_EVENTS.load(Ordering::Relaxed);
            let errors = TOTAL_ERRORS.load(Ordering::Relaxed);
            let eps = total as f64 / elapsed.max(1.0);
            info!(
                "METRICS => total_events={} errors={} eps={:.2}/s uptime={:.1}s",
                total, errors, eps, elapsed
            );
        }
    });
}

/// Backward-compatible default (5s)
pub fn init() {
    init_with_interval(5);
}

/// Record a successfully processed event.
pub fn record_success() {
    TOTAL_EVENTS.fetch_add(1, Ordering::Relaxed);
}

/// Record a failed insert or parse.
pub fn record_error() {
    TOTAL_ERRORS.fetch_add(1, Ordering::Relaxed);
}
