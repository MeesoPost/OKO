use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "plex-pinger", about = "Monitors Plex and qBittorrent, sends Pushover alerts on failure")]
pub struct Config {
    #[clap(long, default_value = "", help = "Plex identity endpoint, e.g. http://192.168.1.10:32400/identity (empty = disabled)")]
    pub plex_url: String,

    #[clap(long, default_value = "", help = "qBittorrent WebUI base URL, e.g. http://192.168.1.10:8080 (empty = disabled)")]
    pub qbit_url: String,

    #[clap(long, default_value = "", help = "NAS check URL — http(s)://... or tcp://host:port, e.g. tcp://192.168.1.20:445 (empty = disabled)")]
    pub nas_url: String,

    #[clap(long, env = "PUSHOVER_TOKEN", help = "Pushover application token")]
    pub pushover_token: String,

    #[clap(long, env = "PUSHOVER_USER", help = "Pushover user key")]
    pub pushover_user: String,

    #[clap(long, default_value = "60", help = "Check interval in seconds")]
    pub interval_seconds: u64,

    #[clap(long, default_value = "5", help = "HTTP request timeout in seconds")]
    pub timeout_seconds: u64,

    #[clap(long, default_value = "2", help = "Consecutive failures before marking a service as down")]
    pub failure_threshold: u32,

    #[clap(long, default_value = "30", help = "Wait this many seconds at startup before first check (lets services settle after host reboot)")]
    pub startup_grace_seconds: u64,
}
