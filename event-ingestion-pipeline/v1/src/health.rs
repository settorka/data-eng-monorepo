#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    Ready,
    Degraded,
    Unready,
}

#[derive(Debug, Clone)]
pub struct HealthState {
    readiness: Readiness,
}

impl HealthState {
    pub fn new(readiness: Readiness) -> Self {
        Self { readiness }
    }

    pub fn readiness(&self) -> Readiness {
        self.readiness
    }
}
