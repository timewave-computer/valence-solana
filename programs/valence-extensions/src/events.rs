//! Minimal event emission

use anchor_lang::prelude::*;

/// Emit a structured event
#[macro_export]
macro_rules! emit_event {
    ($event_type:expr, $($field:ident: $value:expr),*) => {
        msg!("EVENT:{} {}", $event_type, stringify!($($field=$value)*));
    };
}

/// Event builder for complex events
pub struct EventBuilder {
    event_type: String,
    fields: Vec<(String, String)>,
}

impl EventBuilder {
    pub fn new(event_type: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            fields: Vec::new(),
        }
    }
    
    pub fn field(mut self, name: &str, value: impl ToString) -> Self {
        self.fields.push((name.to_string(), value.to_string()));
        self
    }
    
    pub fn emit(self) {
        let fields_str = self.fields
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");
        msg!("EVENT:{} {}", self.event_type, fields_str);
    }
}