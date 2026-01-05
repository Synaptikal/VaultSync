use super::{EmailMessage, EmailProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

/// Mock provider that logs emails to console/stdout
/// Used for development and testing
pub struct MockEmailProvider;

#[async_trait]
impl EmailProvider for MockEmailProvider {
    async fn send_email(&self, message: &EmailMessage) -> Result<()> {
        let attachment_info = message
            .attachment_path
            .as_ref()
            .map(|p| format!(" (Attachment: {})", p))
            .unwrap_or_default();

        tracing::info!(
            "📧 [MOCK EMAIL]\n  To: {}\n  Subject: {}{}\n  Body Preview: {}...",
            message.to,
            message.subject,
            attachment_info,
            message.body.chars().take(200).collect::<String>()
        );
        Ok(())
    }
}

/// SMTP provider using `lettre`
/// Supports Gmail, Outlook, and other standard SMTP servers
///
/// ## Environment Variables
/// - `SMTP_HOST` - SMTP server hostname (required)
/// - `SMTP_PORT` - SMTP port (optional, default: 587)
/// - `SMTP_USERNAME` - SMTP username/email (required)
/// - `SMTP_PASSWORD` - SMTP password or app password (required)
/// - `SMTP_FROM` - From address (optional, defaults to SMTP_USERNAME)
pub struct SmtpEmailProvider {
    mailer: SmtpTransport,
    from_address: String,
}

impl SmtpEmailProvider {
    pub fn new() -> Result<Self> {
        let host = env::var("SMTP_HOST").context("SMTP_HOST not set")?;
        let port: u16 = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587);
        let username = env::var("SMTP_USERNAME").context("SMTP_USERNAME not set")?;
        let password = env::var("SMTP_PASSWORD").context("SMTP_PASSWORD not set")?;
        let from_address = env::var("SMTP_FROM").unwrap_or_else(|_| username.clone());

        let creds = Credentials::new(username, password);

        // Build transport based on port
        // Port 465 = SMTPS (implicit TLS), Port 587 = STARTTLS
        let mailer = if port == 465 {
            SmtpTransport::relay(&host)?
                .port(port)
                .credentials(creds)
                .build()
        } else {
            // Default: STARTTLS on port 587
            SmtpTransport::starttls_relay(&host)?
                .port(port)
                .credentials(creds)
                .build()
        };

        tracing::info!(
            "SMTP Email Provider initialized: {}:{} (from: {})",
            host,
            port,
            from_address
        );

        Ok(Self {
            mailer,
            from_address,
        })
    }
}

#[async_trait]
impl EmailProvider for SmtpEmailProvider {
    async fn send_email(&self, message: &EmailMessage) -> Result<()> {
        // Determine if body is HTML
        let is_html = message.body.trim_start().starts_with("<!DOCTYPE")
            || message.body.trim_start().starts_with("<html")
            || message.body.contains("</html>");

        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(message.to.parse()?)
            .subject(&message.subject)
            .header(if is_html {
                ContentType::TEXT_HTML
            } else {
                ContentType::TEXT_PLAIN
            })
            .body(message.body.clone())?;

        // Clone for move into spawn_blocking
        let mailer = self.mailer.clone();
        let to = message.to.clone();
        let subject = message.subject.clone();

        tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .context("Failed to execute email send task")?
            .context("Failed to send email via SMTP")?;

        tracing::info!("📧 Email sent to {} - Subject: {}", to, subject);
        Ok(())
    }
}

/// Factory to get the appropriate email provider based on environment
///
/// If `SMTP_HOST` is set, attempts to create an SMTP provider.
/// Falls back to MockEmailProvider if SMTP setup fails or isn't configured.
pub fn get_email_provider() -> Box<dyn EmailProvider> {
    if env::var("SMTP_HOST").is_ok() {
        match SmtpEmailProvider::new() {
            Ok(p) => {
                tracing::info!("Using SMTP Email Provider");
                Box::new(p)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize SMTP provider, falling back to Mock: {}",
                    e
                );
                Box::new(MockEmailProvider)
            }
        }
    } else {
        tracing::info!(
            "SMTP not configured, using Mock Email Provider (emails will be logged only)"
        );
        Box::new(MockEmailProvider)
    }
}
