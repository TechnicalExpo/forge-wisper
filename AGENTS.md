# Forge Wisper Engineering Rules

This file is the repository-level source of truth for coding, branching, release, security, and verification rules. These rules apply to every future session and every contributor unless a user explicitly changes them.

## Project Baseline

- Current release version: `0.1.3`.
- Application stack: Tauri 2, Rust 2021, React, TypeScript, Vite, Tailwind CSS, pnpm, and a Cargo workspace.
- Rust is used for audio capture, transcription providers, cleanup, verification, storage, OS integration, hotkeys, and output injection.
- TypeScript is used for UI, view state, frontend API wrappers, and presentation logic.

## Branching And Merge Flow

1. Never develop directly on `main`.
2. Start every change from the latest `main`:

```powershell
git switch main
git pull --ff-only origin main
git switch -c <type>/<short-description>
```

3. Use one focused branch per fix, bug, feature, documentation change, or release task.
4. Recommended branch prefixes are `fix/`, `feat/`, `perf/`, `refactor/`, `docs/`, `test/`, and `chore/`.
5. Keep unrelated changes out of the branch.
6. Commit focused, reviewable changes with Conventional Commit messages.
7. Push the branch and open a PR when collaboration/review is needed.
8. Merge into `main` only after all required checks pass and the change is reviewed.
9. After merge, update local `main`, then create a new branch for the next change.
10. Never force-push or rewrite shared branch history.

## Required Verification Gate

Before every commit claiming a code change is ready, run:

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --filter @forge-wisper/desktop build
```

On Windows, run Rust commands from a Visual Studio Developer environment or ensure Microsoft's MSVC linker precedes Git/Hermes `link.exe` on `PATH`. If Windows Smart App Control blocks generated Cargo build scripts, do not weaken source code or bypass security silently; document the environment blocker and obtain explicit user approval for any security-setting change.

For Rust formatting, check with:

```powershell
cargo fmt --all -- --check
```

Do not reformat unrelated files as part of a focused fix. If the repository has pre-existing formatting drift, report it separately or use a dedicated formatting branch.

## TDD And Debugging

- For behavior changes and bug fixes, reproduce the issue first.
- Trace the root cause before changing production code.
- Add or update a focused regression test where practical.
- Prefer one hypothesis and one minimal change at a time.
- Do not bundle performance refactors with unrelated behavior changes.
- Verify the original symptom after the fix, not only compilation.
- Measure performance before and after optimization when the change is performance-related.

## Coding Style

### Rust

- Use Rust idioms and explicit error handling.
- Prefer `Result`/typed errors over panics in runtime paths.
- Keep real-time audio callbacks small, bounded, and non-blocking.
- Do not hold mutexes across `.await` points.
- Avoid blocking filesystem, registry, or process work on latency-sensitive paths.
- Keep provider, storage, cleanup, verification, output, and UI concerns separated.
- Add `Default` implementations when Clippy requires them and the default is meaningful.
- Preserve platform-specific behavior with explicit `cfg` blocks and tests.

### TypeScript/React

- Keep strict TypeScript enabled; do not weaken compiler settings to silence errors.
- Keep backend/IPC calls typed in `src/lib/tauri.ts`.
- Do not use overlapping `setInterval(async () => ...)` polling.
- Avoid duplicate event subscriptions and redundant full-data reloads.
- Keep business logic out of presentational components where a domain/backend function is more appropriate.
- Preserve keyboard accessibility and responsive behavior.
- Do not add memoization or abstraction without evidence that it solves a measured problem.

### Naming And Structure

- Prefer domain-specific names over generic `utils`, `helpers`, or `common` modules.
- Keep functions focused and files reasonably sized.
- Avoid deep nesting and duplicated business rules.
- Comments should explain non-obvious decisions, not restate code.

## Versioning And Changelog

- Forge Wisper follows Semantic Versioning: `MAJOR.MINOR.PATCH`.
- Every user-visible update must bump the version before merge.
- Bug fixes/performance fixes/documentation-only releases normally bump PATCH.
- Backward-compatible features bump MINOR.
- Breaking changes bump MAJOR.
- Keep these application versions synchronized:
  - root `package.json`
  - `Cargo.toml` workspace package version
  - `apps/desktop/package.json`
  - `apps/desktop/src-tauri/tauri.conf.json`
- Update `CHANGELOG.md` under `Unreleased` or the new version entry.
- Mention migration, privacy, security, installer, or compatibility impact when applicable.
- Do not change dependency versions merely to bump the application version.

## LPGS Release Checklist

Every release/update must review LPGS:

### License

- Confirm the project license remains accurate.
- Review new dependencies and bundled binaries for license compatibility.
- Preserve required copyright, attribution, and notice text.
- Update third-party notices when adding or replacing dependencies.

### Privacy

- Do not persist microphone audio unless explicitly documented and approved.
- Never log API keys, credentials, raw audio, or sensitive transcript data.
- Document cloud-provider data flow and local/offline behavior.
- Review new telemetry, update, network, and storage behavior.

### Governance

- Use the branch/PR/verification flow in this file.
- Update README, CONTRIBUTING, CHANGELOG, and release notes when behavior changes.
- Keep Code of Conduct and contributor guidance respected.
- Record important architectural decisions in `docs/`.

### Security

- Store secrets in OS keyrings, never settings JSON or SQLite.
- Validate external input and URLs before invoking OS commands.
- Review subprocesses, auto-start, global hotkeys, clipboard, file writes, and installer changes.
- Report vulnerabilities privately using `SECURITY.md`.
- Do not disable security controls in code to make builds pass.

## Documentation Rules

- Update README when setup, supported platforms, commands, features, or user-facing behavior changes.
- Update `CONTRIBUTING.md` when developer workflow changes.
- Update `SECURITY.md` when security guarantees, supported versions, or disclosure procedures change.
- Put detailed audits, architecture decisions, and remediation plans under `docs/`.
- Keep examples executable and version references current.

## Do Not

- Do not commit secrets, API keys, credentials, model credentials, or private certificates.
- Do not commit generated build output unless explicitly required by the release process.
- Do not edit or delete user changes unrelated to the current task.
- Do not claim tests, builds, merges, or releases passed without fresh command output.
- Do not merge a branch with failing required checks.
