use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use ureq::Agent;

const IP_PROVIDERS: &[&str] = &[
    "https://api.ipify.org",
    "https://ipv4.icanhazip.com",
    "https://ipv4.seeip.org",
];

/// Three-state result — Unknown means "couldn't determine", don't change state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckResult {
    Up,
    Down,
    Unknown,
}

/// Run a check based on the URL scheme:
///   http:// / https://  → HTTP GET, expect status < 400
///   tcp://host:port      → TCP connect
///   vpn://ISP_IP         → VPN leak check (alert if public IP == ISP IP)
pub fn run(agent: &Agent, url: &str, timeout: Duration) -> CheckResult {
    match url.split_once("://") {
        Some(("http" | "https", _)) => run_http(agent, url),
        Some(("tcp", addr)) => run_tcp(addr, timeout),
        Some(("vpn", isp_ip)) => {
            let Ok(isp_addr) = isp_ip.parse::<IpAddr>() else {
                return CheckResult::Unknown;
            };
            run_vpn_leak(agent, isp_addr)
        }
        _ => CheckResult::Unknown,
    }
}

fn run_http(agent: &Agent, url: &str) -> CheckResult {
    match agent.get(url).call() {
        Ok(r) if r.status() < 400 => CheckResult::Up,
        _ => CheckResult::Down,
    }
}

fn run_tcp(addr: &str, timeout: Duration) -> CheckResult {
    let Some(sock_addr) = addr.to_socket_addrs().ok().and_then(|mut a| a.next()) else {
        return CheckResult::Down;
    };
    if TcpStream::connect_timeout(&sock_addr, timeout).is_ok() {
        CheckResult::Up
    } else {
        CheckResult::Down
    }
}

fn run_vpn_leak(agent: &Agent, isp_ip: IpAddr) -> CheckResult {
    for &url in IP_PROVIDERS {
        let Ok(resp) = agent.get(url).call() else { continue };
        let Ok(body) = resp.into_string() else { continue };
        let Ok(current) = body.trim().parse::<IpAddr>() else { continue };
        return if current == isp_ip { CheckResult::Down } else { CheckResult::Up };
    }
    CheckResult::Unknown
}
