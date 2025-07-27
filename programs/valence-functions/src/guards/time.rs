// Time-based guard implementations
use super::core::GuardFunction;
use crate::Environment;
use anchor_lang::prelude::*;

/// Time window guard for specific periods
#[derive(Clone, Debug)]
pub struct TimeWindowGuard {
    pub start_time: i64,
    pub end_time: i64,
    pub daily_recurring: bool,
}

impl TimeWindowGuard {
    /// Create a one-time window
    pub fn new(start_time: i64, end_time: i64) -> Self {
        Self {
            start_time,
            end_time,
            daily_recurring: false,
        }
    }

    /// Create a daily recurring window
    pub fn daily(start_hour: u8, end_hour: u8) -> Self {
        Self {
            start_time: (start_hour as i64) * 3600,
            end_time: (end_hour as i64) * 3600,
            daily_recurring: true,
        }
    }

    fn is_within_window(&self, current_time: i64) -> bool {
        if self.daily_recurring {
            let seconds_in_day = 86400;
            let time_of_day = current_time % seconds_in_day;
            time_of_day >= self.start_time && time_of_day <= self.end_time
        } else {
            current_time >= self.start_time && current_time <= self.end_time
        }
    }
}

impl GuardFunction for TimeWindowGuard {
    type State = ();

    fn check(&self, _state: &Self::State, _operation: &[u8], env: &Environment) -> Result<bool> {
        Ok(self.is_within_window(env.timestamp))
    }

    fn description(&self) -> &'static str {
        "Time window guard"
    }

    fn compute_cost(&self) -> u64 {
        300
    }
}

/// Rate limiting guard
#[derive(Clone, Debug)]
pub struct RateLimitGuard {
    pub max_operations: u32,
    pub window_duration: i64,
    pub current_count: u32,
    pub window_start: i64,
}

impl RateLimitGuard {
    pub fn new(max_operations: u32, window_duration: i64) -> Self {
        Self {
            max_operations,
            window_duration,
            current_count: 0,
            window_start: 0,
        }
    }

    pub fn would_exceed_limit(&self, current_time: i64) -> bool {
        if current_time >= self.window_start + self.window_duration {
            return false;
        }
        self.current_count >= self.max_operations
    }
}

impl GuardFunction for RateLimitGuard {
    type State = ();

    fn check(&self, _state: &Self::State, _operation: &[u8], env: &Environment) -> Result<bool> {
        Ok(!self.would_exceed_limit(env.timestamp))
    }

    fn description(&self) -> &'static str {
        "Rate limiting guard"
    }

    fn compute_cost(&self) -> u64 {
        400
    }
}