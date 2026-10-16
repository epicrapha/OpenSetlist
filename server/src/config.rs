//! Runtime configuration, parsed once at startup from the environment.

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    /// Set `COOKIE_SECURE=true` in production (HTTPS). Local dev defaults to false.
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;

        let bind_addr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("BIND_ADDR is not a valid socket address: {e}"))?;

        let cookie_secure = matches!(
            std::env::var("COOKIE_SECURE").as_deref(),
            Ok("true") | Ok("1")
        );

        Ok(Self {
            database_url,
            bind_addr,
            cookie_secure,
        })
    }
}
