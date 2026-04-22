# OKO

Service monitoring daemon — watches Plex, qBittorrent, and your NAS, and sends Pushover notifications when something goes down (or comes back up).

## Quick start

1. Copy `.env.example` to `.env` and fill in your [Pushover](https://pushover.net) credentials:

   ```shell
   PUSHOVER_TOKEN=...
   PUSHOVER_USER=...
   ```

2. Edit the URLs in `docker-compose.yml` for your network. Anything you don't want to monitor, leave empty (`--nas-url=`) or remove the line.

3. Start it:

   ```shell
   docker compose up -d --build
   ```

4. Tail the logs:

   ```shell
   docker compose logs -f oko
   ```

## Config

All options are CLI flags (see `docker-compose.yml`):

| Flag | Default | Purpose |
| --- | --- | --- |
| `--plex-url` | _(empty)_ | Plex identity endpoint, e.g. `http://10.0.0.5:32400/identity` |
| `--qbit-url` | _(empty)_ | qBittorrent WebUI, e.g. `http://10.0.0.5:8080` |
| `--nas-url` | _(empty)_ | NAS, `http://...` or `tcp://host:port` (e.g. `tcp://10.0.0.10:445` for SMB) |
| `--isp-ip` | _(unset)_ | Your ISP's public IP. When set, alerts if traffic starts leaking outside ProtonVPN (see below) |
| `--interval-seconds` | `60` | How often to check |
| `--timeout-seconds` | `5` | HTTP/TCP timeout per check |
| `--failure-threshold` | `2` | Consecutive failures before alerting (avoids false alarms) |
| `--startup-grace-seconds` | `30` | Wait time at startup so services can come up after a reboot |

Pushover credentials come from env vars (`PUSHOVER_TOKEN`, `PUSHOVER_USER`).

## What you get

- 🔴 alert when a service goes down
- 🟢 alert when it's back online (with total downtime)
- Re-alerts after 1h, 6h, and 24h if something stays down
- Auto-restart on Docker or host reboot (`restart: unless-stopped`)

## VPN leak detection (optional)

For machines running ProtonVPN natively (e.g. a Windows PC with the ProtonVPN desktop app), OKO can alert you when traffic starts flowing outside the tunnel.

**Setup:**

1. Temporarily disconnect ProtonVPN.
2. Visit <https://api.ipify.org> and note your ISP's public IPv4.
3. Reconnect ProtonVPN.
4. Add the flag to `docker-compose.yml`:

   ```yaml
   - --isp-ip=84.22.123.45
   ```

5. Restart: `docker compose up -d`.

**How it works:**

Each cycle OKO queries your current public IP (via `api.ipify.org`, with `icanhazip.com` and `seeip.org` as fallbacks). If the result matches the IP you configured, traffic is no longer routing through ProtonVPN → 🔴 alert.

If all three IP providers are unreachable for a cycle, OKO marks the result as unknown and does not change state — a temporary network blip will not trigger a false "VPN down" alert.

**Startup sanity check:** at launch, OKO does one VPN check and warns if it looks like VPN is already off (i.e. your current IP already matches `--isp-ip`). This catches the "I pasted the wrong IP" mistake before it generates hours of alerts.

**Maintenance:** your ISP's IP changes rarely (typical cable lease: 6+ months). When it does, you'll see constant "VPN is down" alerts — that's the signal to update the flag.

## Building locally without Docker

```shell
cargo build --release
./target/release/oko --help
```
