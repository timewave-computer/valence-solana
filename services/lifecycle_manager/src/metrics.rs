//! Prometheus metrics for monitoring

use prometheus::{Registry, Counter, Gauge, Histogram, HistogramOpts};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ACCOUNT_REQUESTS_TOTAL: Counter = Counter::new(
        "valence_account_requests_total",
        "Total number of account requests processed"
    ).unwrap();
    
    pub static ref ACCOUNTS_INITIALIZED_TOTAL: Counter = Counter::new(
        "valence_accounts_initialized_total",
        "Total number of accounts successfully initialized"
    ).unwrap();
    
    pub static ref SESSIONS_CREATED_TOTAL: Counter = Counter::new(
        "valence_sessions_created_total",
        "Total number of sessions created"
    ).unwrap();
    
    pub static ref SESSIONS_CONSUMED_TOTAL: Counter = Counter::new(
        "valence_sessions_consumed_total",
        "Total number of sessions consumed"
    ).unwrap();
    
    pub static ref ACTIVE_SESSIONS: Gauge = Gauge::new(
        "valence_active_sessions",
        "Current number of active sessions"
    ).unwrap();
    
    pub static ref ACTIVE_ACCOUNTS: Gauge = Gauge::new(
        "valence_active_accounts",
        "Current number of active accounts"
    ).unwrap();
    
    pub static ref PROGRESSION_RULES_EVALUATED: Counter = Counter::new(
        "valence_progression_rules_evaluated_total",
        "Total number of progression rules evaluated"
    ).unwrap();
    
    pub static ref PROGRESSION_RULES_MATCHED: Counter = Counter::new(
        "valence_progression_rules_matched_total",
        "Total number of progression rules that matched"
    ).unwrap();
    
    pub static ref ACCOUNT_INITIALIZATION_TIME: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "valence_account_initialization_duration_seconds",
            "Time taken to initialize an account"
        )
    ).unwrap();
    
    pub static ref SESSION_LIFETIME: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "valence_session_lifetime_seconds",
            "Lifetime of sessions from creation to consumption"
        )
    ).unwrap();
    
    pub static ref BUNDLE_EXECUTION_TIME: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "valence_bundle_execution_duration_seconds",
            "Time taken to execute a bundle"
        )
    ).unwrap();
}

pub fn init_metrics() -> Registry {
    let registry = Registry::new();
    
    registry.register(Box::new(ACCOUNT_REQUESTS_TOTAL.clone())).unwrap();
    registry.register(Box::new(ACCOUNTS_INITIALIZED_TOTAL.clone())).unwrap();
    registry.register(Box::new(SESSIONS_CREATED_TOTAL.clone())).unwrap();
    registry.register(Box::new(SESSIONS_CONSUMED_TOTAL.clone())).unwrap();
    registry.register(Box::new(ACTIVE_SESSIONS.clone())).unwrap();
    registry.register(Box::new(ACTIVE_ACCOUNTS.clone())).unwrap();
    registry.register(Box::new(PROGRESSION_RULES_EVALUATED.clone())).unwrap();
    registry.register(Box::new(PROGRESSION_RULES_MATCHED.clone())).unwrap();
    registry.register(Box::new(ACCOUNT_INITIALIZATION_TIME.clone())).unwrap();
    registry.register(Box::new(SESSION_LIFETIME.clone())).unwrap();
    registry.register(Box::new(BUNDLE_EXECUTION_TIME.clone())).unwrap();
    
    registry
}