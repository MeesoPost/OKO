use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub url: String,
}

pub struct ServiceState {
    pub healthy: bool,
    pub consecutive_failures: u32,
    pub down_since: Option<Instant>,
    pub realerts_sent: usize,
    pub last_checked: Option<Instant>,
}

impl ServiceState {
    fn new() -> Self {
        Self {
            healthy: true,
            consecutive_failures: 0,
            down_since: None,
            realerts_sent: 0,
            last_checked: None,
        }
    }
}

pub struct Entry {
    pub config: ServiceConfig,
    pub state: ServiceState,
}

impl Entry {
    pub fn new(config: ServiceConfig) -> Self {
        Self { config, state: ServiceState::new() }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ServicesFile {
    #[serde(default)]
    pub services: Vec<ServiceConfig>,
}

pub fn load_services(path: &str) -> Vec<ServiceConfig> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str::<ServicesFile>(&s).ok())
        .map(|f| f.services)
        .unwrap_or_default()
}
