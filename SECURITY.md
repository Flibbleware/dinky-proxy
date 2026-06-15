# Security Policy

## Reporting a Vulnerability

Please report security issues **privately** — do not open a public issue for an
unfixed vulnerability.

- Preferred: GitHub → **Security** tab → **Report a vulnerability** (private
  advisory).

Please include reproduction steps and the affected version/commit. We aim to
acknowledge reports within a few days.

## Security Model

DinkyProxy is a local desktop (tray) app that runs two loopback servers and wraps
an upstream corporate proxy:

- A **local HTTP proxy** on `127.0.0.1` that attaches the configured upstream
  credentials and forwards traffic to the upstream proxy (HTTP CONNECT or
  SOCKS5).
- A **PAC server** on `127.0.0.1` that serves a generated `proxy.pac`.
- System proxy configuration applied via OS tools (`networksetup` on macOS,
  registry/`netsh` on Windows).

Credentials are encrypted at rest (AES-256-GCM) using a 256-bit master key that
is generated once and stored in the OS keychain/credential manager.

### Hardening in place

- Loopback-only listeners (no network exposure).
- Request line, header, and body size limits on the local proxy (DoS guard).
- Timeouts on all upstream connect/handshake operations.
- Generated PAC values are validated to prevent JavaScript injection.
- Config and PAC files are written `0600` (owner-only) on Unix.
- A restrictive Content Security Policy is applied to the webview.
- Subprocess calls use argument arrays (no shell), so no command injection.

## Known, Accepted Trade-offs

These are inherent to the app's purpose and are documented rather than "fixed":

1. **Local processes can use the proxy.** The local proxy does not authenticate
   its clients; any process running as the same user can route traffic through
   it and have the stored upstream credentials attached. This is the intended
   transparent-proxy behavior and is bounded by the loopback binding. Treat the
   machine's local trust boundary accordingly.

2. **Upstream credentials are sent in cleartext to the proxy.** HTTP "Basic"
   auth (base64) and SOCKS5 username/password auth are not encrypted on the wire
   — this is how those proxy protocols work. The upstream proxy is assumed to be
   on a trusted network path (e.g. a corporate LAN). Pointing the app at a remote
   proxy over the open internet would expose credentials on the wire.

3. **Debug builds use a fixed, public encryption key.** Development builds skip
   the keychain and use a hardcoded key so `config.enc` is **not** secure in
   debug builds. Only distribute release builds (`tauri build`, which compiles
   `--release`). Never ship a debug binary.

## Dependencies

Dependencies are audited with `cargo audit` and `pnpm audit`. Some transitive
crates pulled in through the GUI framework's Linux backend carry "unmaintained"
or "unsound" advisories; these are not on the security-critical path for the
supported targets (macOS and Windows). Run the audits before each release.

## Supported Versions

Security fixes are applied to the latest released version.
