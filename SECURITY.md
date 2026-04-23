# Security Policy

## Reporting a Vulnerability

If you discover a security issue in OpenDesk, please **do not open a public
issue**. Instead, email the maintainer directly:

**erenkn@gmail.com** — include `[OpenDesk security]` in the subject.

Please include:

- A description of the issue and its potential impact.
- Steps to reproduce (minimal proof-of-concept if applicable).
- Affected version(s) (`Settings → About` in the app, or the git tag).
- Your suggested fix, if any.

We aim to acknowledge reports within **72 hours** and to ship a fix or
mitigation within **30 days** for high-severity issues. Low-severity issues
may be folded into the next scheduled release.

## Scope

In scope:

- Bluetooth command injection or protocol abuse in `ble/linak.rs` and related
  modules.
- Webview → Rust IPC boundary (capabilities, commands in `src-tauri/src/commands.rs`).
- Updater integrity (signing, endpoint pinning, fallback behaviour).
- macOS entitlements / Gatekeeper / notarization bypasses.
- Build/release pipeline (GitHub Actions secrets handling, supply chain).

Out of scope:

- Findings that require physical access to an unlocked machine.
- Social-engineering scenarios involving tricking the desk owner.
- Denial-of-service against the user's own desk.
- Issues in upstream dependencies already tracked by RustSec / GHSA — please
  report those to the upstream project.

## Supported Versions

The latest release on `main` is the only supported line. Older versions
receive fixes only if the underlying bug is critical and hard to
backport-forward-compatibly.

## Public Disclosure

Once a fix has shipped in a tagged release, we'll acknowledge the reporter
(with permission) in the release notes.
