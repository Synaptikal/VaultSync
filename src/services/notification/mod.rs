pub mod email;
pub mod scheduler;
pub mod sms;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body: String, // HTML or Text
    pub attachment_path: Option<String>,
}

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send_email(&self, message: &EmailMessage) -> Result<()>;
}
