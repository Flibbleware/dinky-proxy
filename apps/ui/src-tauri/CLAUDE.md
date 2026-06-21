# Rust guidance

Applies to the Tauri backend crate (`.rs` files in this directory).

## Error handling

- Always consider error handling deliberately. Avoid `unwrap()`/`expect()` on fallible operations outside tests and genuinely-impossible cases — return `Result` and propagate with `?`, surfacing errors to the caller (and the UI) with enough context to be actionable. Prefer typed errors over stringly-typed ones, and never silently swallow an error.

## Formatting

- Formatting is handled by `cargo fmt`. A Husky `pre-commit` hook (lint-staged) runs `cargo fmt` on the crate when any `.rs` file is staged, so commits are auto-formatted — but edits are not formatted as you go. Run `cargo fmt --manifest-path apps/ui/src-tauri/Cargo.toml` to format manually before committing.

## Security

- When changing Rust code, consider whether the change introduces any security pitfalls. Watch for untrusted input reaching Tauri commands, the filesystem, the network, or a shell (path traversal, command/argument injection), secrets being logged or leaked to the frontend, overly broad capabilities/permissions, and unsafe deserialization. Validate and sanitize at trust boundaries, and call out anything questionable rather than assuming it's fine.
