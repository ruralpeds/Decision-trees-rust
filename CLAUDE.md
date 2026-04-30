# CLAUDE.md — cds-tree-rs Project Configuration

This file configures Claude's behavior for this project.

## Project Context

**Project:** `cds-tree-rs` — Clinical Decision Tree Engine for Medical Decision Support
**Owner:** ruralpeds GitHub organization
**Status:** Sprint 1 — Core Data Model & Evaluation Engine (Weeks 1–2)
**Target Repo:** `https://github.com/ruralpeds/cds-tree-rs`

## Coding Standards

### Rust Edition & Idioms

- **Edition:** 2024 (once stable; currently 2021)
- **MSRV:** 1.75+ (enforced via CI)
- **Code Style:** `rustfmt` with default settings
- **Linting:** `cargo clippy` — deny all warnings except intentional
- **Safety:** `#![forbid(unsafe_code)]` in library crates

### Module Organization

Each crate follows this structure:

```
src/
├── lib.rs              # Module declarations + re-exports
├── error.rs            # Error types
├── model/              # Domain model
│   ├── mod.rs
│   ├── tree.rs
│   ├── node.rs
│   ├── input.rs
│   ├── outcome.rs
│   └── evidence.rs
└── engine/             # Business logic
    ├── mod.rs
    ├── evaluator.rs
    └── validator.rs
```

### Naming Conventions

- **Files:** `snake_case.rs`
- **Types:** `PascalCase`
- **Functions/Methods:** `snake_case`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Modules:** `snake_case` (directory names, file names)

### Documentation

Every public item requires a doc comment:

```rust
/// Brief description (one line).
///
/// Longer explanation with examples if helpful.
///
/// # Examples
///
/// ```
/// let x = function();
/// assert_eq!(x, 42);
/// ```
///
/// # Errors
///
/// Returns `Err(...)` if ...
pub fn function() -> Result<i32, Error> { }
```

### Testing

- **Unit tests:** In the same file, after the implementation, behind `#[cfg(test)]`
- **Integration tests:** In `tests/` directory at workspace root
- **Fixtures:** In `fixtures/` directory with `.json` or `.yaml` extension
- **Coverage target:** > 75% for library code, > 50% for API handlers

## Type Safety Principles

1. **No stringly-typed APIs** — use `enum` instead of `&str` for variant selection
2. **Explicit error types** — `enum` errors, not `String` errors
3. **No panics in libraries** — `Result<T, E>` for errors
4. **Validation at type construction** — invariants enforced by constructors, not at call sites
5. **Serde-derived from the start** — all domain types derive `Serialize`/`Deserialize`

## Async/Concurrency

- **Async runtime:** Tokio (full feature set)
- **Async functions:** Use `async` keyword, not trait objects
- **Channels:** `tokio::sync` (mpsc, watch, broadcast)
- **No blocking calls:** Use `tokio::task::spawn_blocking` for sync operations
- **Structured concurrency:** Use `tokio::task::JoinSet` for fan-out/fan-in

## Dependencies

### Policy

- Minimize direct dependencies (prefer pulling via `tokio` ecosystem)
- Prefer `no_std`-compatible crates in `cds-tree-core` (enable `alloc`)
- Version policy: `^X.Y.Z` for stable crates, `^0.Y.Z` with bump-on-minor for pre-1.0
- Audit: `cargo-audit` weekly; zero known vulnerabilities in CI

### Approved Core Dependencies

```
serde + serde_json
uuid
chrono
thiserror
anyhow (binary only; use thiserror in libraries)
tokio
tower / tower-http
axum
sqlx
tracing / tracing-subscriber
```

## Git Workflow

1. **Branch naming:** `sprint-{N}`, `feature/{name}`, `fix/{issue}`, `docs/{topic}`
2. **Commit messages:** Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`)
3. **PR reviews:** Require 1 approval from the ruralpeds org before merge
4. **CI:** All checks must pass; auto-merge on approval
5. **Tags:** Semantic versioning at release (`v0.1.0`, `v0.1.1-alpha.1`)

## Sprint Execution

### Current Sprint (Sprint 1: Core Model & Evaluator)

**Goal:** Rust type system fully encodes decision trees; edge condition evaluator passes all tests.

**Deliverables:**
- [x] `cds-tree-core` crate scaffolded
- [x] All `NodeInput` variants implemented
- [x] `DecisionNode`, `ClinicalDecisionTree` types complete
- [x] `EdgeCondition` evaluator with 6+ test cases per variant
- [x] `TreeValidator` with cycle detection, reachability checks
- [ ] Three fixture trees (neonatal-respiratory-distress, pediatric-fever, neonatal-jaundice)
- [ ] 50+ passing unit tests
- [ ] `cargo test --lib` and `cargo clippy` passing with no warnings

**Definition of Done:** Run `cargo test --lib` and observe all 50+ tests passing with 0 warnings.

### Next Sprint (Sprint 2: REST API Foundation)

Scheduled for weeks 3–4. Will focus on Axum scaffolding, JWT auth, and health endpoints.

## Tools & Environment

### Local Development (Mac Studio or Linux)

```bash
# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install stable

# Clone repo
git clone https://github.com/ruralpeds/cds-tree-rs
cd cds-tree-rs

# Run tests
cargo test

# Format and lint
cargo fmt
cargo clippy -- -D warnings

# Build release
cargo build --release
```

### Docker Development

```bash
docker-compose up -d postgres prometheus grafana
cargo test
cargo build --release
```

## Code Review Checklist

Before submitting a PR, verify:

- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test --all` passes
- [ ] New public types have doc comments
- [ ] New functions have `#[cfg(test)]` test cases
- [ ] Commit messages follow Conventional Commits
- [ ] No `unwrap()` or `expect()` in library code (use `?` operator)
- [ ] All `TODO` comments are tied to a GitHub issue

## Deployment

### Docker Images

Built automatically via GitHub Actions on tag:

```bash
docker pull ghcr.io/ruralpeds/cds-tree-rs-api:v0.1.0
docker pull ghcr.io/ruralpeds/cds-tree-rs-api:latest
```

Environment variables (see `.env.example`):

```
DATABASE_URL=postgres://user:pass@host:5432/cds_tree
RUST_LOG=info
JWT_PUBLIC_KEY_PEM=...
```

## Resources

- **Project Plan:** [cds-tree-rs-project-plan.md](../cds-tree-rs-project-plan.md)
- **Rust Book:** https://doc.rust-lang.org/book/
- **Tokio Tutorial:** https://tokio.rs/tokio/tutorial
- **Axum Docs:** https://docs.rs/axum/latest/
- **HL7 CDS Hooks:** https://cds-hooks.hl7.org/
- **FHIR R4:** https://hl7.org/fhir/R4/

## Contact

- **Team Lead:** Timothy Hartzog (timothy@ruralpeds.org)
- **GitHub Issues:** Use labels `sprint-1`, `bug`, `documentation`, `enhancement`
- **Slack:** `#cds-tree-rs` channel in ruralpeds workspace
