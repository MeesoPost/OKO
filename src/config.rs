use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "oko", about = "Lightweight service monitor with Pushover alerts")]
pub struct Config {
    /// Path to services config file
    #[arg(long, env = "OKO_CONFIG", default_value = "services.toml")]
    pub config_path: String,

    /// Check interval in seconds
    #[arg(long, default_value = "60")]
    pub interval_seconds: u64,

    /// HTTP/TCP request timeout in seconds
    #[arg(long, default_value = "5")]
    pub timeout_seconds: u64,

    /// Consecutive failures before alerting
    #[arg(long, default_value = "2")]
    pub failure_threshold: u32,

    /// Wait this many seconds at startup before first check
    #[arg(long, default_value = "30")]
    pub startup_grace_seconds: u64,

    /// Pushover application token
    #[arg(long, env = "PUSHOVER_TOKEN")]
    pub pushover_token: String,

    /// Pushover user key
    #[arg(long, env = "PUSHOVER_USER")]
    pub pushover_user: String,
}
