use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use ureq::Agent;

pub fn check(agent: &Agent, url: &str, timeout: Duration) -> bool {
    match url.split_once("://") {
        Some(("tcp", addr)) => check_tcp(addr, timeout),
        _ => check_http(agent, url),
    }
}

fn check_http(agent: &Agent, url: &str) -> bool {
    agent
        .get(url)
        .call()
        .map(|r| r.status() == 200)
        .unwrap_or(false)
}

fn check_tcp(addr: &str, timeout: Duration) -> bool {
    let Some(sock_addr) = addr.to_socket_addrs().ok().and_then(|mut a| a.next()) else {
        return false;
    };
    TcpStream::connect_timeout(&sock_addr, timeout).is_ok()
}
