//! Application lifecycle management.
//!
//! Handles graceful startup and shutdown of the inference pool.
//!
//! ## Example
//!
//! ```ignore
//! use inference_pool::lifecycle::{LifecycleManager, ShutdownReason};
//!
//! let lifecycle = LifecycleManager::new();
//!
//! // Register cleanup handlers
//! lifecycle.on_shutdown(|| async {
//!     println!("Shutting down...");
//! });
//!
//! // Wait for shutdown signal
//! lifecycle.wait_for_shutdown().await;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

// ============================================================================
// Shutdown Reason
// ============================================================================

/// Reason for shutdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownReason {
    /// SIGINT/SIGTERM received
    Signal,
    /// Admin requested shutdown
    AdminRequest,
    /// Fatal error occurred
    FatalError(String),
    /// Health check failed
    HealthCheckFailed,
    /// Dependency unavailable
    DependencyUnavailable(String),
    /// Scheduled maintenance
    Maintenance,
}

impl std::fmt::Display for ShutdownReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signal => write!(f, "Signal received"),
            Self::AdminRequest => write!(f, "Admin requested shutdown"),
            Self::FatalError(e) => write!(f, "Fatal error: {}", e),
            Self::HealthCheckFailed => write!(f, "Health check failed"),
            Self::DependencyUnavailable(d) => write!(f, "Dependency unavailable: {}", d),
            Self::Maintenance => write!(f, "Scheduled maintenance"),
        }
    }
}

// ============================================================================
// Lifecycle State
// ============================================================================

/// Application lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleState {
    /// Starting up
    Starting,
    /// Running and serving traffic
    Running,
    /// Draining connections (not accepting new)
    Draining,
    /// Shutting down
    ShuttingDown,
    /// Stopped
    Stopped,
}

impl Default for LifecycleState {
    fn default() -> Self {
        Self::Starting
    }
}

// ============================================================================
// Shutdown Handler
// ============================================================================

type ShutdownHandlerFn =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

struct ShutdownHandler {
    name: String,
    priority: u32, // Lower = runs first
    handler: ShutdownHandlerFn,
}

// ============================================================================
// Lifecycle Manager
// ============================================================================

/// Manages application lifecycle
pub struct LifecycleManager {
    /// Current state
    state: RwLock<LifecycleState>,

    /// Start time
    start_time: DateTime<Utc>,

    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<ShutdownReason>,

    /// Shutdown handlers
    handlers: RwLock<Vec<ShutdownHandler>>,

    /// Drain timeout in seconds
    drain_timeout_secs: u64,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            state: RwLock::new(LifecycleState::Starting),
            start_time: Utc::now(),
            shutdown_tx,
            handlers: RwLock::new(Vec::new()),
            drain_timeout_secs: 30,
        }
    }

    /// Set drain timeout
    pub fn with_drain_timeout(mut self, secs: u64) -> Self {
        self.drain_timeout_secs = secs;
        self
    }

    /// Get current state
    pub async fn state(&self) -> LifecycleState {
        *self.state.read().await
    }

    /// Check if running
    pub async fn is_running(&self) -> bool {
        *self.state.read().await == LifecycleState::Running
    }

    /// Check if accepting new connections
    pub async fn accepting_connections(&self) -> bool {
        matches!(
            *self.state.read().await,
            LifecycleState::Starting | LifecycleState::Running
        )
    }

    /// Transition to running state
    pub async fn mark_running(&self) {
        let mut state = self.state.write().await;
        *state = LifecycleState::Running;
        tracing::info!("Application is now running");
    }

    /// Register a shutdown handler
    pub async fn on_shutdown<F, Fut>(&self, name: impl Into<String>, priority: u32, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handler = ShutdownHandler {
            name: name.into(),
            priority,
            handler: Arc::new(move || Box::pin(handler())),
        };

        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
        // Sort by priority
        handlers.sort_by_key(|h| h.priority);
    }

    /// Subscribe to shutdown events
    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<ShutdownReason> {
        self.shutdown_tx.subscribe()
    }

    /// Trigger shutdown
    pub async fn shutdown(&self, reason: ShutdownReason) {
        tracing::info!(reason = %reason, "Initiating shutdown");

        // Update state to draining
        {
            let mut state = self.state.write().await;
            if *state == LifecycleState::ShuttingDown || *state == LifecycleState::Stopped {
                return; // Already shutting down
            }
            *state = LifecycleState::Draining;
        }

        // Notify subscribers
        let _ = self.shutdown_tx.send(reason.clone());

        // Wait for drain timeout
        tracing::info!(
            timeout_secs = self.drain_timeout_secs,
            "Draining connections"
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(self.drain_timeout_secs)).await;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = LifecycleState::ShuttingDown;
        }

        // Run shutdown handlers
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            tracing::debug!(handler = %handler.name, "Running shutdown handler");
            let start = std::time::Instant::now();
            (handler.handler)().await;
            tracing::debug!(
                handler = %handler.name,
                duration_ms = %start.elapsed().as_millis(),
                "Shutdown handler completed"
            );
        }

        // Final state
        {
            let mut state = self.state.write().await;
            *state = LifecycleState::Stopped;
        }

        tracing::info!("Shutdown complete");
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&self) -> ShutdownReason {
        let mut rx = self.subscribe_shutdown();

        // Also listen for OS signals
        let signal_task = async {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).unwrap();
                let mut sigint = signal(SignalKind::interrupt()).unwrap();

                tokio::select! {
                    _ = sigterm.recv() => ShutdownReason::Signal,
                    _ = sigint.recv() => ShutdownReason::Signal,
                }
            }

            #[cfg(not(unix))]
            {
                tokio::signal::ctrl_c().await.ok();
                ShutdownReason::Signal
            }
        };

        tokio::select! {
            reason = rx.recv() => {
                reason.unwrap_or(ShutdownReason::Signal)
            }
            reason = signal_task => {
                reason
            }
        }
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        (Utc::now() - self.start_time).num_seconds().max(0) as u64
    }

    /// Get start time
    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Startup Validation
// ============================================================================

/// Startup validation result
#[derive(Debug, Clone)]
pub struct StartupValidation {
    /// Component name
    pub name: String,

    /// Whether validation passed
    pub passed: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Startup validator
pub struct StartupValidator {
    validations: Vec<StartupValidation>,
}

impl StartupValidator {
    /// Create a new startup validator
    pub fn new() -> Self {
        Self {
            validations: Vec::new(),
        }
    }

    /// Validate a component
    pub async fn validate<F, Fut>(&mut self, name: impl Into<String>, check: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        let name = name.into();
        let start = std::time::Instant::now();

        let result = check().await;
        let duration_ms = start.elapsed().as_millis() as u64;

        let validation = match result {
            Ok(()) => {
                tracing::info!(component = %name, duration_ms, "Startup validation passed");
                StartupValidation {
                    name,
                    passed: true,
                    error: None,
                    duration_ms,
                }
            }
            Err(e) => {
                tracing::error!(component = %name, error = %e, "Startup validation failed");
                StartupValidation {
                    name,
                    passed: false,
                    error: Some(e),
                    duration_ms,
                }
            }
        };

        self.validations.push(validation);
    }

    /// Get all validations
    pub fn validations(&self) -> &[StartupValidation] {
        &self.validations
    }

    /// Check if all validations passed
    pub fn all_passed(&self) -> bool {
        self.validations.iter().all(|v| v.passed)
    }

    /// Get failed validations
    pub fn failures(&self) -> Vec<&StartupValidation> {
        self.validations.iter().filter(|v| !v.passed).collect()
    }
}

impl Default for StartupValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_reason_display() {
        assert!(ShutdownReason::Signal.to_string().contains("Signal"));
        assert!(ShutdownReason::FatalError("test".to_string())
            .to_string()
            .contains("test"));
    }

    #[tokio::test]
    async fn test_lifecycle_manager_states() {
        let manager = LifecycleManager::new();

        assert_eq!(manager.state().await, LifecycleState::Starting);

        manager.mark_running().await;
        assert_eq!(manager.state().await, LifecycleState::Running);
        assert!(manager.is_running().await);
    }

    #[tokio::test]
    async fn test_lifecycle_accepting_connections() {
        let manager = LifecycleManager::new();

        assert!(manager.accepting_connections().await);

        manager.mark_running().await;
        assert!(manager.accepting_connections().await);
    }

    #[tokio::test]
    async fn test_startup_validator() {
        let mut validator = StartupValidator::new();

        validator
            .validate("good_component", || async { Ok(()) })
            .await;

        validator
            .validate("bad_component", || async { Err("Failed".to_string()) })
            .await;

        assert_eq!(validator.validations().len(), 2);
        assert!(!validator.all_passed());
        assert_eq!(validator.failures().len(), 1);
    }

    #[tokio::test]
    async fn test_shutdown_handlers() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let manager = Arc::new(LifecycleManager::new().with_drain_timeout(0));
        let handler_called = Arc::new(AtomicBool::new(false));

        let called = handler_called.clone();
        manager
            .on_shutdown("test", 0, move || {
                let called = called.clone();
                async move {
                    called.store(true, Ordering::SeqCst);
                }
            })
            .await;

        manager.mark_running().await;
        manager.shutdown(ShutdownReason::Signal).await;

        assert!(handler_called.load(Ordering::SeqCst));
        assert_eq!(manager.state().await, LifecycleState::Stopped);
    }

    #[test]
    fn test_uptime() {
        let manager = LifecycleManager::new();
        assert!(manager.uptime_secs() >= 0);
    }
}


