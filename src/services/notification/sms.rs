use anyhow::{Context, Result};
use async_trait::async_trait;
use std::env;

#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, to: &str, body: &str) -> Result<()>;
}

pub struct MockSmsProvider;

#[async_trait]
impl SmsProvider for MockSmsProvider {
    async fn send_sms(&self, to: &str, body: &str) -> Result<()> {
        tracing::info!("📱 [MOCK SMS] To: {}, Body: {}", to, body);
        Ok(())
    }
}

// Placeholder for Twilio or other providers
pub struct TwilioSmsProvider {
    client: reqwest::Client,
    account_sid: String,
    auth_token: String,
    from_number: String,
}

impl TwilioSmsProvider {
    pub fn new() -> Result<Self> {
        let account_sid = env::var("TWILIO_ACCOUNT_SID").context("TWILIO_ACCOUNT_SID not set")?;
        let auth_token = env::var("TWILIO_AUTH_TOKEN").context("TWILIO_AUTH_TOKEN not set")?;
        let from_number = env::var("TWILIO_FROM_NUMBER").context("TWILIO_FROM_NUMBER not set")?;

        Ok(Self {
            client: reqwest::Client::new(),
            account_sid,
            auth_token,
            from_number,
        })
    }
}

#[async_trait]
impl SmsProvider for TwilioSmsProvider {
    async fn send_sms(&self, to: &str, body: &str) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let params = [("To", to), ("From", &self.from_number), ("Body", body)];

        let resp = self
            .client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await?;

        if resp.status().is_success() {
            tracing::info!("Sent SMS to {}", to);
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Twilio API Error {}: {}", status, text))
        }
    }
}

pub fn get_sms_provider() -> Box<dyn SmsProvider> {
    if env::var("TWILIO_ACCOUNT_SID").is_ok() {
        match TwilioSmsProvider::new() {
            Ok(p) => Box::new(p),
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize Twilio provider, falling back to Mock: {}",
                    e
                );
                Box::new(MockSmsProvider)
            }
        }
    } else {
        Box::new(MockSmsProvider)
    }
}
