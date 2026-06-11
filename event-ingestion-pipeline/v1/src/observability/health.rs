use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Liveness = process health only.
/// Must stay true even if dependencies fail.
#[derive(Debug, Clone, Serialize)]
pub struct Liveness {
    pub ok: bool,
    pub service: &'static str,
    pub unix_time_ms: u128,
}

/// Readiness = safe to accept traffic.
/// Fails closed on dependency failure.
#[derive(Debug, Clone, Serialize)]
pub struct Readiness {
    pub ok: bool,
    pub service: &'static str,
    pub broker_ok: bool,
    pub scylla_ok: bool,
    pub accepting_traffic: bool,
    pub unix_time_ms: u128,
}

/// Shared state with lock-free reads in hot path.
#[derive(Debug, Clone)]
pub struct HealthState {
    broker_ok: Arc<AtomicBool>,
    scylla_ok: Arc<AtomicBool>,
    accepting_traffic: Arc<AtomicBool>,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthState {
    /// Start fail-closed to prevent unsafe acceptance.
    pub fn new() -> Self {
        Self {
            broker_ok: Arc::new(AtomicBool::new(false)),
            scylla_ok: Arc::new(AtomicBool::new(false)),
            accepting_traffic: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Dependency signals update readiness.
    pub fn set_broker_ok(&self, ok: bool) {
        self.broker_ok.store(ok, Ordering::Release);
        self.recompute_acceptance();
    }

    pub fn set_scylla_ok(&self, ok: bool) {
        self.scylla_ok.store(ok, Ordering::Release);
        self.recompute_acceptance();
    }

    /// Allows explicit fail-closed override.
    pub fn set_accepting_traffic(&self, accepting: bool) {
        self.accepting_traffic.store(accepting, Ordering::Release);
    }

    pub fn broker_ok(&self) -> bool {
        self.broker_ok.load(Ordering::Acquire)
    }

    pub fn scylla_ok(&self) -> bool {
        self.scylla_ok.load(Ordering::Acquire)
    }

    pub fn accepting_traffic(&self) -> bool {
        self.accepting_traffic.load(Ordering::Acquire)
    }

    /// Ready only if all safety conditions hold.
    pub fn is_ready(&self) -> bool {
        self.broker_ok() && self.scylla_ok() && self.accepting_traffic()
    }

    pub fn liveness(&self, service: &'static str) -> Liveness {
        Liveness {
            ok: true,
            service,
            unix_time_ms: unix_time_ms(),
        }
    }

    pub fn readiness(&self, service: &'static str) -> Readiness {
        Readiness {
            ok: self.is_ready(),
            service,
            broker_ok: self.broker_ok(),
            scylla_ok: self.scylla_ok(),
            accepting_traffic: self.accepting_traffic(),
            unix_time_ms: unix_time_ms(),
        }
    }

    /// Keeps readiness consistent with dependencies.
    fn recompute_acceptance(&self) {
        let safe = self.broker_ok() && self.scylla_ok();
        self.accepting_traffic.store(safe, Ordering::Release);
    }
}

/// Fixed probe interval to bound background work.
pub const DEFAULT_HEALTH_PROBE_INTERVAL: Duration = Duration::from_secs(5);

fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_fail_closed() {
        let h = HealthState::new();
        assert!(!h.is_ready());
    }

    #[test]
    fn requires_all_dependencies() {
        let h = HealthState::new();
        h.set_broker_ok(true);
        assert!(!h.is_ready());
        h.set_scylla_ok(true);
        assert!(h.is_ready());
    }

    #[test]
    fn dependency_loss_fails_readiness() {
        let h = HealthState::new();
        h.set_broker_ok(true);
        h.set_scylla_ok(true);
        assert!(h.is_ready());
        h.set_scylla_ok(false);
        assert!(!h.is_ready());
    }

    #[test]
    fn override_blocks_traffic() {
        let h = HealthState::new();
        h.set_broker_ok(true);
        h.set_scylla_ok(true);
        assert!(h.is_ready());
        h.set_accepting_traffic(false);
        assert!(!h.is_ready());
    }
}