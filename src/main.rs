mod checker;
mod config;
mod notifier;

use clap::Parser;
use config::Config;
use std::thread;
use std::time::{Duration, Instant};
use ureq::{Agent, AgentBuilder};

struct Service {
    name: &'static str,
    url: String,
    healthy: bool,
    consecutive_failures: u32,
    down_since: Option<Instant>,
}

impl Service {
    fn new(name: &'static str, url: String) -> Self {
        Self {
            name,
            url,
            healthy: true,
            consecutive_failures: 0,
            down_since: None,
        }
    }
}

fn format_duration(seconds: u64) -> String {
    match seconds {
        s if s < 60 => format!("{} seconden", s),
        s if s < 3600 => match (s / 60, s % 60) {
            (m, 0) => format!("{} minuten", m),
            (m, sec) => format!("{} min {} sec", m, sec),
        },
        s if s < 86400 => match (s / 3600, (s % 3600) / 60) {
            (h, 0) => format!("{} uur", h),
            (h, m) => format!("{} uur {} min", h, m),
        },
        s => match (s / 86400, (s % 86400) / 3600) {
            (d, 0) => format!("{} dagen", d),
            (d, h) => format!("{} dagen {} uur", d, h),
        },
    }
}

fn main() {
    let config = Config::parse();

    let agent: Agent = AgentBuilder::new()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .build();

    println!(
        "plex-pinger started — checking every {}s (alert after {} consecutive failures)",
        config.interval_seconds, config.failure_threshold
    );
    let mut services: Vec<Service> = [
        ("Plex", &config.plex_url),
        ("qBittorrent", &config.qbit_url),
        ("NAS", &config.nas_url),
    ]
    .into_iter()
    .filter_map(|(name, url)| {
        let url = url.trim();
        if url.is_empty() {
            println!("  {}: (disabled)", name);
            None
        } else {
            println!("  {}: {}", name, url);
            Some(Service::new(name, url.to_string()))
        }
    })
    .collect();

    if services.is_empty() {
        eprintln!("No services configured — pass at least one URL. Exiting.");
        std::process::exit(1);
    }

    let interval = Duration::from_secs(config.interval_seconds);
    let check_timeout = Duration::from_secs(config.timeout_seconds);

    if config.startup_grace_seconds > 0 {
        println!(
            "waiting {}s for services to settle before first check",
            config.startup_grace_seconds
        );
        thread::sleep(Duration::from_secs(config.startup_grace_seconds));
    }

    loop {
        for svc in &mut services {
            let healthy = checker::check(&agent, &svc.url, check_timeout);

            if healthy {
                svc.consecutive_failures = 0;
                if !svc.healthy {
                    svc.healthy = true;
                    let duration_str = svc
                        .down_since
                        .take()
                        .map(|t| format_duration(t.elapsed().as_secs()))
                        .unwrap_or_else(|| "onbekend".to_string());
                    let message = format!("🟢 {} is weer online (was {} down)", svc.name, duration_str);
                    println!("{}", message);
                    notify(&agent, &config, &message, "recovery");
                }
            } else {
                svc.consecutive_failures = svc.consecutive_failures.saturating_add(1);
                if svc.healthy && svc.consecutive_failures >= config.failure_threshold {
                    svc.healthy = false;
                    svc.down_since = Some(Instant::now());
                    let message = format!(
                        "🔴 {} is down (na {} mislukte checks)",
                        svc.name, svc.consecutive_failures
                    );
                    eprintln!("{}", message);
                    notify(&agent, &config, &message, "alert");
                }
            }
        }

        thread::sleep(interval);
    }
}

fn notify(agent: &Agent, config: &Config, message: &str, kind: &str) {
    if let Err(e) = notifier::send_pushover(
        agent,
        &config.pushover_token,
        &config.pushover_user,
        message,
    ) {
        eprintln!("Failed to send Pushover {}: {}", kind, e);
    }
}
