//! Email channel.
//!
//! SEAM / STUB: no SMTP client is wired yet (no `lettre` dependency, and SMTP
//! credentials should come from Vault, which isn't set up). For now this logs
//! the email it *would* send, tagged distinctly so it's obvious in logs and
//! easy to grep. When ready:
//!   1. add `lettre` to Cargo.toml,
//!   2. read SMTP creds from config/Vault into `EmailChannel`,
//!   3. replace the body of `send` with a real `lettre` transport send.
//! Nothing else in the system changes — callers dispatch `Alert`s the same way.

use crate::core::errors::app_error::AppError;
use crate::modules::alerts::application::channel::AlertChannel;
use crate::modules::alerts::domain::alert::Alert;
use async_trait::async_trait;

pub struct EmailChannel {
    /// The From: address real sends will use. Carried now so the wiring is
    /// already in place when SMTP is added.
    pub from_address: String,
}

impl EmailChannel {
    pub fn new(from_address: impl Into<String>) -> Self {
        Self {
            from_address: from_address.into(),
        }
    }
}

#[async_trait]
impl AlertChannel for EmailChannel {
    fn name(&self) -> &'static str {
        "email"
    }

    async fn send(&self, alert: &Alert) -> Result<(), AppError> {
        // Only attempt "email" when there's a recipient; otherwise no-op.
        let Some(to) = alert.recipient.as_deref() else {
            return Ok(());
        };

        // TODO(email): replace with lettre transport send.
        tracing::info!(
            target: "alerts::email",
            from = %self.from_address,
            to = %to,
            subject = %alert.title,
            "EMAIL (stub — not actually sent): {}",
            alert.body
        );

        Ok(())
    }
}
