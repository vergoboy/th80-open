# Phase 14 — Test and Platform Report

## Environment

- OS: Arch Linux x86_64, kernel 7.2.11-arch1-1.
- Rust: rustc 1.99.1 (58cf8f9c3 2026-10-13).
- Cargo: 1.99.1 (5f94df478 2026-08-27).
- GUI backend smoke: native X11.

## Verification results

| Command | Result |
|---|---|
| cargo fmt --all --check | PASS |
| cargo test --workspace --locked | PASS — 107 tests, 0 failures |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS |
| cargo build --workspace --release --locked | PASS |
| cargo doc --workspace --no-deps --locked | PASS |
| cargo tree --locked -p th80-gui -e normal forbidden dependency scan | PASS — no HIDAPI/USB/probe/Vial/QMK/RMK match |
| ldd target/release/th80-gui unresolved dependency scan | PASS — no “not found” entries |

## Test breakdown

| Package/target | Tests |
|---|---:|
| th80-cli unit tests | 3 |
| th80-cli offline workflow integration tests | 6 |
| th80-core unit tests | 30 |
| th80-core schema compatibility integration test | 1 |
| th80-core repository process tests | 5 |
| th80-gui state/service/screen tests | 15 |
| th80-probe library tests | 26 |
| th80-probe binary tests | 6 |
| th80-probe offline protocol integration tests | 12 |
| Root th80-hid-research parser tests | 3 |
| **Total** | **107** |

New focused coverage includes:

- CLI rename/duplicate workflow, fresh identity and unknown top-level extension preservation.
- Core validation of non-empty layer and macro display names.
- GUI duplicate profile identity/configuration/tag/extension preservation.
- GUI profile-specific editor-buffer reset on profile switches.
- GUI stale-save conflict detection and explicit reload state transition.
- Confirmed macro deletion updates only the in-memory draft, selects a remaining macro, and leaves the profile dirty.
- Macro step reordering and out-of-range rejection.
- UTC timestamp formatting.

Existing core tests continue to cover schema-v1 compatibility, unknown extension preservation, unknown nested-field rejection, optional lighting semantics, import/export, atomic replacement, crash/failure cases, stale writes, lock contention and concurrent processes.

## Native GUI smoke test

The release GUI binary was launched with an isolated synthetic profile directory and forced to use X11:

- Command target: target/release/th80-gui.
- Profile store: a fresh /tmp/th80-phase14-gui-smoke-final.* directory.
- Result: process remained alive until the 8-second timeout (exit 124), with no startup error output.
- The synthetic profile directory remained empty.

This confirms that the native X11 application starts and remains running. It does **not** constitute manual interaction testing: no pointer/keyboard workflow, profile editing, native dialog, accessibility tree or screen-reader behavior was exercised.

## Platform status

| Platform | Build/test status in this phase | Notes |
|---|---|---|
| Linux x86_64 | Verified | Workspace tests, Clippy, release build, docs and X11 GUI launch smoke passed. Wayland-specific interaction was not confirmed. |
| Windows | Cross-target checked; not run natively here | `cargo check --locked -p th80-gui --target x86_64-pc-windows-msvc` passed from Linux. Existing repository workflow includes GUI tests, Clippy, release build and docs on windows-latest; hosted/native result remains pending. |
| macOS | Not run natively here | Existing repository workflow was extended to include GUI tests, Clippy, release build and docs on macos-latest. Hosted result remains pending. |

The CI workflow change is configuration only; this phase did not push code or trigger hosted GitHub Actions. Native Windows/macOS filesystem and GUI behavior must not be described as verified until those jobs execute successfully.

## Test isolation and safety

- CLI integration tests set TH80_PROFILE_DIR to unique temporary directories.
- GUI state/service tests use tempfile directories.
- The GUI smoke test used a fresh /tmp profile directory, which remained empty.
- No test used the user's real profile directory.
- Workspace probe tests use synthetic parser fixtures and pure in-memory protocol builders; the transport remains disabled.
- No physical device was enumerated or opened, no HID/USB report was sent, and no keyboard state was changed.
- No updater or firmware was executed or modified.

## Limitations

- No pointer-level GUI test harness was available.
- No manual end-to-end GUI editing/import/export interaction was claimed.
- No formal screen-reader/accessibility audit was performed despite enabling AccessKit integration.
- Windows/macOS GUI builds and native behavior remain pending their CI/native runners.

See [application audit](PHASE14_APPLICATION_AUDIT.md), [implementation report](PHASE14_IMPLEMENTATION_REPORT.md) and [hardware integration readiness](PHASE14_HARDWARE_INTEGRATION_READINESS.md).
