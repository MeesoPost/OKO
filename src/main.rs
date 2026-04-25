mod checker;
mod config;
mod notifier;
mod service;

use checker::CheckResult;
use clap::Parser;
use config::Config;
use service::{load_services, Entry};
use std::thread;
use std::time::{Duration, Instant};
use ureq::AgentBuilder;

const REALERT_AFTER: &[Duration] = &[
    Duration::from_secs(60 * 60),
    Duration::from_secs(6 * 60 * 60),
    Duration::from_secs(24 * 60 * 60),
];

fn main() {
    let config = Config::parse();

    if config.pushover_token.trim().is_empty() || config.pushover_user.trim().is_empty() {
        eprintln!("PUSHOVER_TOKEN or PUSHOVER_USER is empty — refusing to start.");
        std::process::exit(1);
    }

    let initial = load_services(&config.config_path);
    if initial.is_empty() {
        eprintln!(
            "No services in {} — edit the file and restart.",
            config.config_path
        );
        std::process::exit(1);
    }

    println!("OKO — monitoring {} service(s):", initial.len());
    for svc in &initial {
        println!("  {} → {}", svc.name, svc.url);
    }

    let mut entries: Vec<Entry> = initial.into_iter().map(Entry::new).collect();

    let agent = AgentBuilder::new()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .build();

    println!(
        "Checking every {}s, alert after {} consecutive failures",
        config.interval_seconds, config.failure_threshold
    );

    if config.startup_grace_seconds > 0 {
        println!("Waiting {}s grace period…", config.startup_grace_seconds);
        thread::sleep(Duration::from_secs(config.startup_grace_seconds));
    }

    let interval = Duration::from_secs(config.interval_seconds);
    let timeout = Duration::from_secs(config.timeout_seconds);

    loop {
        let cycle_start = Instant::now();

        for entry in &mut entries {
            let result = checker::run(&agent, &entry.config.url, timeout);
            entry.state.last_checked = Some(Instant::now());

            let msg = match result {
                CheckResult::Up => record_success(entry),
                CheckResult::Down => record_failure(entry, config.failure_threshold),
                CheckResult::Unknown => None,
            };

            if let Some(msg) = msg {
                let is_recovery = result == CheckResult::Up;
                if is_recovery { println!("{}", msg); } else { eprintln!("{}", msg); }
                let kind = if is_recovery { "recovery" } else { "alert" };
                if let Err(e) = notifier::send_pushover(&agent, &config.pushover_token, &config.pushover_user, &msg) {
                    eprintln!("Pushover {} failed: {}", kind, e);
                }
            }
        }

        let elapsed = cycle_start.elapsed();
        if elapsed < interval {
            thread::sleep(interval - elapsed);
        } else {
            eprintln!("warning: check cycle took {} (longer than interval)", format_duration(elapsed));
        }
    }
}

fn record_success(entry: &mut Entry) -> Option<String> {
    entry.state.consecutive_failures = 0;
    if entry.state.healthy { return None; }
    entry.state.healthy = true;
    entry.state.realerts_sent = 0;
    let duration = entry.state.down_since.take()
        .map(|t| format_duration(t.elapsed()))
        .unwrap_or_else(|| "unknown".into());
    Some(format!("🟢 {} is back online (was down for {})", entry.config.name, duration))
}

fn record_failure(entry: &mut Entry, threshold: u32) -> Option<String> {
    entry.state.consecutive_failures = entry.state.consecutive_failures.saturating_add(1);

    if entry.state.healthy && entry.state.consecutive_failures >= threshold {
        entry.state.healthy = false;
        entry.state.down_since = Some(Instant::now());
        return Some(format!(
            "🔴 {} is down (after {} failed checks)",
            entry.config.name, entry.state.consecutive_failures
        ));
    }

    if !entry.state.healthy {
        if let Some(down_since) = entry.state.down_since {
            let elapsed = down_since.elapsed();
            if let Some(&next) = REALERT_AFTER.get(entry.state.realerts_sent) {
                if elapsed >= next {
                    entry.state.realerts_sent += 1;
                    return Some(format!(
                        "🔴 {} is still down — offline for {}",
                        entry.config.name, format_duration(elapsed)
                    ));
                }
            }
        }
    }

    None
}

fn format_duration(d: Duration) -> String {
    let total = d.as_secs();
    let parts: Vec<String> = [
        (total / 86_400, "day", "days"),
        ((total % 86_400) / 3600, "hour", "hours"),
        ((total % 3600) / 60, "minute", "minutes"),
        (total % 60, "second", "seconds"),
    ]
    .iter()
    .filter(|(n, _, _)| *n > 0)
    .take(2)
    .map(|(n, s, p)| format!("{} {}", n, if *n == 1 { s } else { p }))
    .collect();

    if parts.is_empty() { "0 seconds".into() } else { parts.join(" ") }
}
