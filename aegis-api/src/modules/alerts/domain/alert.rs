//! Alerts domain.
//!
//! An `Alert` is a notification the system wants to *deliver* (push) through one
//! or more channels — distinct from `admin::security::SecurityAlert`, which is a
//! *derived read-model* shown on the dashboard. This module is the delivery
//! side; the admin module is the query side.

use serde::Serialize;
use std::collections::BTreeMap;

/// Severity of an alert. Mirrors the audit `SecuritySeverity` vocabulary so the
/// two map cleanly, but is owned by this module to keep it decoupled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Low => "low",
            AlertSeverity::Medium => "medium",
            AlertSeverity::High => "high",
            AlertSeverity::Critical => "critical",
        }
    }
}

/// A deliverable alert.
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    /// Machine-readable kind, e.g. "refresh_replay_detected", "password_reset".
    pub kind: String,
    pub severity: AlertSeverity,
    /// Human-readable one-line summary.
    pub title: String,
    /// Longer body / message (e.g. an email body or a reset link line).
    pub body: String,
    /// Optional structured context (user id, ip, etc.).
    pub metadata: BTreeMap<String, String>,
    /// Optional recipient (email address) for channels that target a user.
    pub recipient: Option<String>,
}

impl Alert {
    /// Convenience constructor for a minimal alert.
    pub fn new(
        kind: impl Into<String>,
        severity: AlertSeverity,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            severity,
            title: title.into(),
            body: body.into(),
            metadata: BTreeMap::new(),
            recipient: None,
        }
    }

    /// Builder: attach a recipient (for email-style channels).
    pub fn to_recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipient = Some(recipient.into());
        self
    }

    /// Builder: add a metadata key/value.
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}
