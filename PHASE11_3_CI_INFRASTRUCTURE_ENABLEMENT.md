# Phase 11.3 — CI Infrastructure Enablement

**Status: PARTIALLY COMPLETE — local Git/CI preparation is complete; native Windows/macOS execution and publication are blocked pending an authorized repository destination.**

## 1. Executive summary

This phase investigated the repository identity, nearby Git metadata, GitHub account, Codeberg availability, publication safety and the existing native platform workflow. No suitable existing GitHub or Codeberg destination could be established. No commit, remote, repository creation, push or publication was performed.

The workspace initially had no `.git` metadata. After confirming there was no recoverable TH80 Git repository, I initialized a new local repository on branch `main` with **zero commits, zero staged files and no remote**. This creates no fabricated history and leaves all project files available for review. The existing CI workflow was hardened with read-only repository permissions and a job timeout. `.gitignore` was extended to reduce accidental inclusion of local environment and private-key files.

The three offline crates passed the workflow's package-scoped verification commands on the native Linux host: 82 tests passed, zero failed. The previous Phase 11.2 full-workspace Linux evidence remains 85 passed, zero failed. Windows remains `COMPILES ONLY`; macOS remains `UNVERIFIED`. No GitHub Actions run was triggered because no verified destination exists and publication was not authorized.

## 2. Initial repository state

Workspace: `/home/vergoboy/Projects/projects/th80/th80-open/`

At inspection:

- `git rev-parse --show-toplevel` failed: the workspace and its parents were not inside a Git repository.
- No `.git` directory or `.git` worktree file existed in the workspace.
- No Git remote or commit history was available.
- No matching TH80 Git bundle, patch series or mailbox backup was found under the searched Projects tree.
- `Cargo.toml`, all four package manifests, `Cargo.lock`, CI workflow, README, platform reports and repository tests were present.
- `target/` contained approximately 1.3 GiB of local build output and was already excluded by `.gitignore`.

The local Projects tree contained 145 Git metadata directories. The only Git repository in the sibling TH80 project folder was `entropy-audit`, whose origin is `https://github.com/ergohaven/entropy.git` and whose current commit concerns an unrelated project. It was explicitly rejected as TH80 history or a project remote. No other TH80/EPOMAKER/keyboard-named Git repository or Git metadata was found in the searched Projects tree or home-directory Git metadata scan.

## 3. GitHub and Codeberg identity findings

### GitHub

The GitHub CLI is installed (`gh 2.102.1`) and authenticated to the personal account `vergoboy`. The account lookup reports a user account. `gh auth status` reports scopes `gist`, `read:org`, `repo`, and `workflow`; no token or credential value was emitted or copied into this report.

Read-only account inventory returned 43 repositories, including private repositories. None had a TH80/EPOMAKER/keyboard name or description matching this project. Searches for `TH80 EPOMAKER`, `th80-open`, `EPOMAKER replacement` and `th80 profile manager` did not identify a repository belonging to this workspace. Public projects returned by broad search were unrelated codebases and were not treated as a destination.

The `repo` scope is the relevant GitHub OAuth scope for repository operations and appears sufficient for creating a repository under the authenticated personal account, subject to GitHub/account policy. This capability was not tested with a mutating API request. No repository was created. The `workflow` scope is present, but it does not create a repository destination by itself.

### Codeberg

No Codeberg CLI (`tea`) or configured Codeberg credential helper/session was found. Public Codeberg repository API lookups were attempted but did not return usable responses within the request timeouts; web access to Codeberg search was also blocked. Therefore, no Codeberg account ownership or matching project repository could be verified. This is an access limitation, not evidence that no Codeberg repository exists.

### Documented remotes

The project files contain upstream/reference URLs for crates.io, Vial/QMK and other research references, but no canonical GitHub/Codeberg origin for this project. No origin URL was inferred from the folder name or account name.

**Repository destination conclusion:** no existing remote is verified as belonging to this project. The user must identify an existing destination or explicitly authorize creation of a new one, including owner/name and visibility.

## 4. Publication-readiness and secret audit

The workspace inventory contained 79 files outside `target/`, with no symlinks, empty files or executable files. The only non-`text/*` MIME classification was `crates/th80-core/tests/fixtures/phase10_profile_v1.json`; it is the small synthetic Phase 10 compatibility profile fixture, not user data. No binary firmware packages, installers or driver archives were found inside the workspace.

The following checks were performed:

- Sensitive filename scan for `.env*`, private-key containers, credential/secret names, database vaults and SSH private-key filenames: no matches.
- Pattern scan for PEM private-key headers, GitHub tokens, AWS access keys, Slack tokens, Google API keys, bearer credentials, credential-bearing URLs and long generic secret assignments: zero matches.
- User-home absolute path scan: no `/home/vergoboy`, `/home/arman`, macOS user-home or Windows user-home paths were found in the workspace.
- Symlink and executable-file scan: zero matches.
- Profile/capture extension scan: only the synthetic Phase 10 JSON compatibility fixture was found; no profile directory, JSONL, PCAP, binary dump or raw capture file was present.
- The `target/` directory is ignored and contains local build artifacts only.

No dedicated `gitleaks`, `trufflehog` or `detect-secrets` binary is installed. The secret scan is therefore a documented pattern-based audit, not a guarantee against every possible secret encoding or unknown credential format. A dedicated secret scanner should be run before the first commit/push if one becomes available.

The project includes one intentional hardware-derived test fixture: a 20-byte interface-2 HID report descriptor captured passively from Linux sysfs. Its fixture README documents its provenance and SHA-256 and explicitly distinguishes it from Feature traffic or a packet capture. It is a descriptor fixture, not a user's device dump or profile. The repository also contains static hardware/firmware research documentation. Those files are part of the declared project scope, but should receive the normal content/license review before public publication.

Several historical research notes mention temporary `/tmp` analysis paths and vendor PDB paths such as `C:\Users\Windows\...`; these are artifact references, not the current user's home path, and the referenced analysis files/PDB are not included. They were left unchanged to preserve historical reports. The parent TH80 folder contains vendor firmware archives/installers outside this workspace; they were not copied or included.

No real user profile data or credentials were identified. No ambiguous file was deleted.

## 5. `.gitignore` and local Git initialization decision

The original `.gitignore` excluded `/target/` and `*.log`. It was extended with exclusions for `.env`/`.env.*` (while allowing `.env.example`), common private-key files/containers (`*.pem`, `*.key`, `*.p12`, `*.pfx`, `id_rsa`, `id_ed25519`), OS metadata (`.DS_Store`, `Thumbs.db`) and editor swap files (`*.swp`). The new rules were checked with `git check-ignore` after initialization.

After confirming no `.git` metadata existed and completing the inventory/secret scan, a local repository was initialized with `git init --initial-branch=main`.

Current Git state:

- Branch: `main`
- Commits: zero
- Staged files: zero
- Remotes: zero
- Existing Git history: none; none was reconstructed or fabricated.
- All project files remain unstaged for review.

This is a local initialization only. It neither identifies a publication destination nor enables Actions until an authorized remote and commit are provided.

## 6. CI workflow audit

File: `.github/workflows/repository-platform.yml`

The workflow declares `push`, `pull_request` and `workflow_dispatch` triggers. Its matrix uses native GitHub-hosted runner labels `ubuntu-latest`, `windows-latest` and `macos-latest`, with `runs-on: ${{ matrix.os }}`. It installs the Rust `stable` toolchain using `dtolnay/rust-toolchain@stable`.

The workflow is a single job, so there are no inter-job dependencies. `fail-fast: false` allows remaining operating-system jobs to continue after another matrix job fails. There are no step-level conditions, `continue-on-error` settings or explicit skip rules. Command failures propagate as job failures. Every Cargo command uses `--locked`. A toolchain-recording step prints the full `rustc` version, Cargo version and active rustup toolchain into each native runner's log; GitHub's job header provides the runner image metadata.

The existing scope deliberately targets the three offline crates (`th80-core`, `th80-cli`, `th80-probe`) for tests, Clippy, release build and documentation. The root `th80ctl` package is a separate legacy Linux HIDAPI research executable and is not expanded into the cross-platform repository matrix. `cargo fmt --all` still checks formatting across the whole workspace.

No system package installation is needed by the selected offline crates. Their dependency boundaries are serialization crates and local `th80-core`; no HIDAPI, USB library, network client or async runtime is pulled into the core/CLI dependency tree. `Cargo.lock` is preserved and every Cargo step is locked.

The workflow was hardened with:

- `permissions: contents: read` to avoid granting write permissions to the workflow token.
- `timeout-minutes: 30` on the matrix job to bound hung jobs.
- A toolchain-recording step so each eventual native run logs the exact Rust and Cargo versions used.

There is no artifact upload step because the workflow currently needs only test/build logs; GitHub Actions log retention follows repository/account defaults. No custom artifact retention policy is configured. Runner image details are supplied by GitHub in each actual Actions run, but no run exists yet to record them.

## 7. Exact workflow commands

The workflow currently runs:

```bash
cargo fmt --all -- --check
cargo test --locked -p th80-core -p th80-cli -p th80-probe
cargo clippy --locked -p th80-core -p th80-cli -p th80-probe --all-targets -- -D warnings
cargo build --locked --release -p th80-core -p th80-cli -p th80-probe
cargo doc --locked -p th80-core -p th80-cli -p th80-probe --no-deps
```

The test/lint/build/doc commands are package-scoped equivalents of the full-workspace commands so that the Linux-only HIDAPI research package is not built or run on Windows/macOS. This preserves the complete test coverage of the three offline crates without expanding the platform matrix into hardware research code. No tests are conditionally skipped by the workflow itself; platform-gated tests execute when compiled for their native OS.

## 8. Native runner execution evidence

No GitHub Actions run could be started. Although the GitHub CLI is authenticated and has `repo`/`workflow` scopes, there is no verified repository destination or remote. The user has not authorized creation of a new repository, publication, commit or push. Therefore:

- Workflow run ID/URL: unavailable.
- Commit SHA: unavailable; the local repository has no commits.
- GitHub runner image/version: unavailable; no Actions job ran.
- Native Windows test execution: not performed.
- Native macOS test execution: not performed.

The local native Linux run below validates the workflow's package-scoped commands only. It is not an Actions run and must not be represented as hosted CI evidence.

## 9. Linux native results

Environment:

- OS/kernel: `Linux Archlinux 7.2.11-arch1-1 #1 SMP PREEMPT_DYNAMIC Fri, 16 Oct 2026 00:28:58 +0000 x86_64 GNU/Linux`
- Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
- Cargo: `cargo 1.99.1 (5f94df478 2026-08-27)`
- Target: `x86_64-unknown-linux-gnu`
- Runner: local native Linux workstation, not GitHub-hosted.

The exact package-scoped test command passed:

```text
cargo test --locked -p th80-core -p th80-cli -p th80-probe
```

| Test target | Passed |
|---|---:|
| CLI unit tests | 3 |
| CLI integration tests | 5 |
| Core unit tests | 24 |
| Phase 10 compatibility test | 1 |
| Cross-process repository integration tests | 5 |
| Probe library tests | 26 |
| Probe binary tests | 6 |
| Offline protocol integration tests | 12 |
| **Total** | **82** |

Result: **82 passed, 0 failed, 0 ignored**. The two crate doc-test suites contained zero tests.

All remaining workflow commands also passed locally:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --locked -p th80-core -p th80-cli -p th80-probe` | PASS — 82 passed |
| `cargo clippy --locked -p th80-core -p th80-cli -p th80-probe --all-targets -- -D warnings` | PASS |
| `cargo build --locked --release -p th80-core -p th80-cli -p th80-probe` | PASS |
| `cargo doc --locked -p th80-core -p th80-cli -p th80-probe --no-deps` | PASS |

Phase 11.2 previously recorded a separate native Linux full-workspace run of 85 passed, 0 failed, including three legacy root research parser tests. That earlier evidence remains valid; this phase did not rerun the full root workspace because no Rust source changed and the CI matrix intentionally excludes the root HIDAPI package.

## 10. Windows and macOS status

| Platform | Status | Evidence |
|---|---|---|
| Linux | VERIFIED | Native package-scoped workflow commands passed; 82 tests. Phase 11.2 also recorded 85 full-workspace tests. |
| Windows | COMPILES ONLY | Phase 11.2 MSVC-target test compilation passed. No native Windows test process has run. |
| macOS | UNVERIFIED | No native macOS compilation or test process has run. |

The Windows `ReplaceFileW` replacement and sharing-denial test still require execution on `windows-latest`. macOS existing-file replacement, parent-directory synchronization, metadata and subprocess behavior still require execution on `macos-latest`. Linux results are not substituted for either platform.

## 11. Workflow changes and validation

Exactly two existing files were modified:

1. `.gitignore` — added environment/private-key/OS/editor exclusions.
2. `.github/workflows/repository-platform.yml` — added read-only token permissions, a 30-minute job timeout and explicit Rust/Cargo version logging.

The workflow YAML parsed successfully. Structural checks confirmed all three native matrix labels, `runs-on` from the matrix, the five command categories, the toolchain-recording step, no conditional skips and no `continue-on-error`. The `.gitignore` behavior was verified using `git check-ignore`. No Cargo manifest, `Cargo.lock`, source code or existing phase report was modified.

## 12. Failures, limitations and outstanding authorization

No native test failure occurred because no Windows or macOS tests were executed. Codeberg public search was inaccessible/time-limited. No suitable existing repository destination was found in the authenticated GitHub account or local TH80 project tree.

The remaining path is:

1. The user identifies an existing GitHub/Codeberg destination or explicitly authorizes creating a new repository. For creation, the owner, repository name and visibility must be confirmed. The current GitHub CLI account has the `repo` scope, but repository creation has deliberately not been tested by mutation.
2. The user explicitly authorizes the initial commit and push. Local files are currently unstaged; no history exists. The first commit would be a new baseline, not a continuation of historical Git history.
3. After an authorized remote is configured and the approved commit is pushed, dispatch `.github/workflows/repository-platform.yml` or use its push/PR trigger. Retain the Actions run URL/ID, commit SHA, runner image/version, Rust version, all five step logs, test counts and skipped/failed test summary for each OS.
4. Do not mark Windows or macOS verified until their native jobs actually complete successfully and the OS-specific replacement/concurrency/crash tests execute.

No remote was added, no repository was created, no commit was made, and no files were pushed or published.

## 13. Dependencies and scope

- Third-party dependencies added: none.
- `Cargo.lock` changed: no.
- Rust source/API/schema/CLI behavior changed: no.
- Desktop UI, database, async runtime or network client added: no.
- Hardware transport contract changed: no.
- Phase 12 work started: no.

## 14. Hardware and user-data safety statement

No physical keyboard or receiver was used in this phase. No HID handle was opened. No HID Feature GET/SET, HID input/output report transfer, USB control transfer, USB endpoint communication, device configuration read/write, firmware communication, flashing/updating, bootloader operation, pairing, calibration, reset, experimental device command or transport activation occurred. CI test commands exercise synthetic fixtures and isolated temporary directories only. The existing passive probe and legacy read-only research code were not executed against hardware. The opt-in input-capture path was not activated.

No real user profile directory was accessed or modified. No user profile data was found in the workspace. The local build cache under `target/` was not included in the Git repository because it is ignored. Historical reports from Phase 5 through Phase 11.2 were preserved. No ambiguous file was deleted.

## 15. Final status

Phase 11.3 has established a local, reviewable Git starting point and hardened the native CI workflow, but it cannot obtain Windows/macOS runtime evidence until the user authorizes a repository destination and the initial commit/push path. Publication remains **NOT PERFORMED**. The local repository has no commit or remote. Stop here pending that decision; do not create a remote, commit, push, publish or start Phase 12 without explicit authorization.
