use prometheus::{Counter, Gauge, Histogram, Registry};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// ArchGuard Enterprise: circuit-breaker, prometheus, empathy_ratio, rhythm detector
pub struct ArchGuard {
    // Circuit breaker
    circuit_open: Arc<AtomicBool>,
    failure_count: Arc<AtomicU64>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u64,
    reset_timeout: Duration,

    // Prometheus metrics
    registry: Registry,
    request_counter: Counter,
    error_counter: Counter,
    latency_histogram: Histogram,
    empathy_ratio: Gauge,

    // Rhythm detector (0.038 Hz = ~26.3 seconds period)
    rhythm_detector: RhythmDetector,

    // Empathy ratio
    empathy_ratio_value: Arc<RwLock<f64>>,
}

impl ArchGuard {
    pub fn new() -> Self {
        let registry = Registry::new();

        let request_counter = Counter::new("archguard_requests_total", "Total number of requests")
            .expect("Failed to create counter");

        let error_counter = Counter::new("archguard_errors_total", "Total number of errors")
            .expect("Failed to create counter");

        let latency_histogram = Histogram::with_opts(prometheus::HistogramOpts::new(
            "archguard_latency_seconds",
            "Request latency in seconds",
        ))
        .expect("Failed to create histogram");

        let empathy_ratio = Gauge::new("archguard_empathy_ratio", "Empathy ratio (0.0 - 1.0)")
            .expect("Failed to create gauge");

        registry
            .register(Box::new(request_counter.clone()))
            .unwrap();
        registry.register(Box::new(error_counter.clone())).unwrap();
        registry
            .register(Box::new(latency_histogram.clone()))
            .unwrap();
        registry.register(Box::new(empathy_ratio.clone())).unwrap();

        Self {
            circuit_open: Arc::new(AtomicBool::new(false)),
            failure_count: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold: 10,
            reset_timeout: Duration::from_secs(30),
            registry,
            request_counter,
            error_counter,
            latency_histogram,
            empathy_ratio,
            rhythm_detector: RhythmDetector::new(0.038), // 0.038 Hz
            empathy_ratio_value: Arc::new(RwLock::new(0.5)),
        }
    }

    /// Execute with circuit breaker protection
    pub async fn execute<F, T>(&self, f: F) -> Result<T, ArchGuardError>
    where
        F: std::future::Future<Output = Result<T, ArchGuardError>>,
    {
        // Check circuit breaker
        if self.circuit_open.load(Ordering::Acquire) {
            // Try to reset
            if self.should_reset().await {
                self.reset_circuit().await;
            } else {
                return Err(ArchGuardError::CircuitOpen);
            }
        }

        let start = Instant::now();
        self.request_counter.inc();

        match f.await {
            Ok(result) => {
                // Success - reset failure count
                self.failure_count.store(0, Ordering::Release);
                let latency = start.elapsed().as_secs_f64();
                self.latency_histogram.observe(latency);
                Ok(result)
            }
            Err(e) => {
                // Failure
                self.error_counter.inc();
                let count = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;

                {
                    let mut last_failure = self.last_failure_time.write().await;
                    *last_failure = Some(Instant::now());
                }

                if count >= self.failure_threshold {
                    self.circuit_open.store(true, Ordering::Release);
                }

                Err(e)
            }
        }
    }

    async fn should_reset(&self) -> bool {
        let last_failure = self.last_failure_time.read().await;
        if let Some(time) = *last_failure {
            time.elapsed() >= self.reset_timeout
        } else {
            false
        }
    }

    async fn reset_circuit(&self) {
        self.circuit_open.store(false, Ordering::Release);
        self.failure_count.store(0, Ordering::Release);
    }

    /// Update empathy ratio (0.0 - 1.0)
    pub async fn update_empathy_ratio(&self, ratio: f64) {
        let clamped = ratio.max(0.0).min(1.0);
        {
            let mut value = self.empathy_ratio_value.write().await;
            *value = clamped;
        }
        self.empathy_ratio.set(clamped);
    }

    /// Get current empathy ratio
    pub async fn get_empathy_ratio(&self) -> f64 {
        *self.empathy_ratio_value.read().await
    }

    /// Check if circuit breaker is open
    pub fn is_circuit_open(&self) -> bool {
        self.circuit_open.load(Ordering::Acquire)
    }

    /// Update rhythm detector
    pub fn update_rhythm(&mut self, timestamp: f64) {
        self.rhythm_detector.update(timestamp);
    }

    /// Get rhythm phase (0.0 - 1.0)
    pub fn get_rhythm_phase(&self) -> f64 {
        self.rhythm_detector.get_phase()
    }

    /// Get Prometheus registry for metrics export
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl Default for ArchGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Rhythm Detector: detects 0.038 Hz rhythm (~26.3 seconds)
struct RhythmDetector {
    frequency: f64, // 0.038 Hz
    period: f64,    // 1 / frequency
    last_update: f64,
    phase: f64,
}

impl RhythmDetector {
    fn new(frequency: f64) -> Self {
        Self {
            frequency,
            period: 1.0 / frequency,
            last_update: 0.0,
            phase: 0.0,
        }
    }

    fn update(&mut self, timestamp: f64) {
        if self.last_update > 0.0 {
            let delta = timestamp - self.last_update;
            self.phase = (self.phase + delta / self.period) % 1.0;
        }
        self.last_update = timestamp;
    }

    fn get_phase(&self) -> f64 {
        self.phase
    }
}

#[derive(Debug, Clone)]
pub enum ArchGuardError {
    CircuitOpen,
    ExecutionFailed(String),
    Timeout,
}

impl std::fmt::Display for ArchGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchGuardError::CircuitOpen => write!(f, "Circuit breaker is open"),
            ArchGuardError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ArchGuardError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for ArchGuardError {}
