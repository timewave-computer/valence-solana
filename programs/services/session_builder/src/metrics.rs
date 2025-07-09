//! Metrics and monitoring for the Session Builder service

use anyhow::Result;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use prometheus::{
    Counter, Histogram, IntGauge, Opts, Registry, TextEncoder, Encoder,
};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, error};

/// Global metrics instance
pub static METRICS: LazyLock<Arc<Metrics>> = LazyLock::new(|| Arc::new(Metrics::new()));

/// Metrics collection for the Session Builder service
pub struct Metrics {
    registry: Registry,
    
    // Counters
    accounts_created_total: Counter,
    accounts_failed_total: Counter,
    events_processed_total: Counter,
    events_failed_total: Counter,
    
    // Histograms
    account_creation_duration: Histogram,
    event_processing_duration: Histogram,
    
    // Gauges
    active_account_creations: IntGauge,
    service_uptime_seconds: IntGauge,
}

impl Metrics {
    /// Create a new Metrics instance
    pub fn new() -> Self {
        let registry = Registry::new();
        
        // Create metrics
        let accounts_created_total = Counter::with_opts(
            Opts::new("session_builder_accounts_created_total", "Total number of accounts created")
        ).unwrap();
        
        let accounts_failed_total = Counter::with_opts(
            Opts::new("session_builder_accounts_failed_total", "Total number of failed account creations")
        ).unwrap();
        
        let events_processed_total = Counter::with_opts(
            Opts::new("session_builder_events_processed_total", "Total number of events processed")
        ).unwrap();
        
        let events_failed_total = Counter::with_opts(
            Opts::new("session_builder_events_failed_total", "Total number of failed event processing")
        ).unwrap();
        
        let account_creation_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "session_builder_account_creation_duration_seconds",
                "Time taken to create accounts"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0])
        ).unwrap();
        
        let event_processing_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "session_builder_event_processing_duration_seconds", 
                "Time taken to process events"
            ).buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0])
        ).unwrap();
        
        let active_account_creations = IntGauge::with_opts(
            Opts::new("session_builder_active_account_creations", "Number of active account creation operations")
        ).unwrap();
        
        let service_uptime_seconds = IntGauge::with_opts(
            Opts::new("session_builder_service_uptime_seconds", "Service uptime in seconds")
        ).unwrap();
        
        // Register metrics
        registry.register(Box::new(accounts_created_total.clone())).unwrap();
        registry.register(Box::new(accounts_failed_total.clone())).unwrap();
        registry.register(Box::new(events_processed_total.clone())).unwrap();
        registry.register(Box::new(events_failed_total.clone())).unwrap();
        registry.register(Box::new(account_creation_duration.clone())).unwrap();
        registry.register(Box::new(event_processing_duration.clone())).unwrap();
        registry.register(Box::new(active_account_creations.clone())).unwrap();
        registry.register(Box::new(service_uptime_seconds.clone())).unwrap();
        
        Metrics {
            registry,
            accounts_created_total,
            accounts_failed_total,
            events_processed_total,
            events_failed_total,
            account_creation_duration,
            event_processing_duration,
            active_account_creations,
            service_uptime_seconds,
        }
    }
    
    /// Record successful account creation
    pub fn record_account_creation_success(&self, duration: Duration) {
        self.accounts_created_total.inc();
        self.account_creation_duration.observe(duration.as_secs_f64());
    }
    
    /// Record failed account creation
    pub fn record_account_creation_failure(&self, duration: Duration) {
        self.accounts_failed_total.inc();
        self.account_creation_duration.observe(duration.as_secs_f64());
    }
    
    /// Record event processing
    pub fn record_event_processed(&self, duration: Duration) {
        self.events_processed_total.inc();
        self.event_processing_duration.observe(duration.as_secs_f64());
    }
    
    /// Record failed event processing
    pub fn record_event_failed(&self, duration: Duration) {
        self.events_failed_total.inc();
        self.event_processing_duration.observe(duration.as_secs_f64());
    }
    
    /// Increment active account creations
    pub fn inc_active_creations(&self) {
        self.active_account_creations.inc();
    }
    
    /// Decrement active account creations
    pub fn dec_active_creations(&self) {
        self.active_account_creations.dec();
    }
    
    /// Update service uptime
    pub fn update_uptime(&self, uptime_seconds: i64) {
        self.service_uptime_seconds.set(uptime_seconds);
    }
    
    /// Record event received
    pub fn record_event_received(&self) {
        self.events_processed_total.inc();
    }
    
    /// Record successful account creation
    pub fn record_account_created(&self) {
        self.accounts_created_total.inc();
    }
    
    /// Record failed account creation
    pub fn record_account_failed(&self) {
        self.accounts_failed_total.inc();
    }
    
    /// Get metrics as Prometheus text format
    pub fn export(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

/// Metrics server for exposing Prometheus metrics
pub struct MetricsServer {
    port: u16,
}

impl MetricsServer {
    /// Create a new metrics server
    pub fn new(port: u16) -> Self {
        Self { port }
    }
    
    /// Start the metrics server
    pub async fn start(&self) -> Result<()> {
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
            );
        
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        info!("Metrics server listening on port {}", self.port);
        
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("Metrics server error: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Shutdown the metrics server
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down metrics server");
        // In a real implementation, you'd store the server handle and shut it down properly
        Ok(())
    }
}

/// Handler for metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    match METRICS.export() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to export metrics".to_string())
        }
    }
}

/// Handler for health check endpoint
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
} 