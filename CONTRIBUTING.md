# Contributing to Decision-Trees-Rust

Welcome! This guide helps you contribute to the clinical decision tree engine.

## Code of Conduct

- Respect all contributors
- Inclusive, professional communication
- Focus on code quality and patient safety

## Getting Started

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/ruralpeds/Decision-trees-rust.git
cd Decision-trees-rust

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install additional tools
cargo install cargo-watch      # Auto-reload on changes
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-audit      # Security audits

# Start development stack
docker-compose up -d

# Verify setup
cargo test --lib
```

### Project Structure

```
Decision-trees-rust/
├── crates/
│   ├── cds-tree-core/          # Core decision tree engine (no_std, WASM-ready)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs        # EvalError, ValidationError
│   │   │   ├── model/          # Tree, Node, Input, Outcome, Evidence
│   │   │   └── engine/         # Evaluator, Validator
│   │   └── tests/
│   ├── cds-tree-api/           # REST API (Axum/Tokio)
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── router.rs       # Route definitions
│   │   │   ├── handlers/       # HTTP handlers
│   │   │   ├── middleware/     # Auth, tracing, CORS
│   │   │   ├── state.rs        # AppState, config
│   │   │   └── error.rs        # AppError, ApiResult
│   │   └── tests/
│   ├── cds-tree-storage/       # PostgreSQL persistence
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models.rs       # Database row types, request/response DTOs
│   │   │   ├── error.rs
│   │   │   └── repo/           # Repositories: trees, nodes, sessions, audit
│   │   ├── migrations/         # SQLx migrations
│   │   └── tests/
│   ├── cds-tree-fhir/          # FHIR R4 + CDS Hooks
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models.rs       # CDS Hooks v2.0, FHIR models
│   │   │   ├── adapters/       # Prefill, export
│   │   │   └── hooks/          # Service builders
│   │   └── tests/
│   ├── cds-tree-audit/         # Audit logging (placeholder)
│   └── cds-tree-wasm/          # WebAssembly (placeholder)
├── migrations/                  # Database migrations
├── fixtures/                    # Example trees (JSON)
├── docker-compose.yml
├── Dockerfile
├── Makefile
├── .github/workflows/          # GitHub Actions CI/CD
└── docs/                       # Additional documentation
```

## Development Workflow

### 1. Pick an Issue

```bash
# Browse open issues
# https://github.com/ruralpeds/Decision-trees-rust/issues

# Comment to claim the issue
# "I'll work on this"
```

### 2. Create Feature Branch

```bash
# Create branch from main
git checkout main
git pull origin main
git checkout -b feat/your-feature-name

# Branch naming conventions:
# feat/     - New feature
# fix/      - Bug fix
# refactor/ - Code cleanup
# docs/     - Documentation
# test/     - Test additions
# perf/     - Performance improvement
```

### 3. Make Changes

```bash
# Start by reading the relevant crate's README or ARCHITECTURE.md

# Make small, focused commits
git add .
git commit -m "Short description (imperative mood)"
# Bad:  "Added support for X"
# Good: "Add support for X"

# Commits should be logical units (one feature per commit)
# Example commits:
# - "Add FhirPrefillAdapter for observation extraction"
# - "Implement TreeValidator with 14-point validation"
# - "Update router with new audit endpoints"
```

### 4. Code Quality Checks

**Before committing, run:**

```bash
# Format code
cargo fmt

# Lint
cargo clippy --all-targets --all-features

# Security audit
cargo audit

# Run tests
cargo test --lib
cargo test --doc

# Type checking (no compilation)
cargo check --all-features
```

**Use Makefile for convenience:**

```bash
make check-all     # Runs all quality checks
make test          # Run all tests
make format        # Format all code
```

### 5. Write Tests

**For every new feature, add tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_basic() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = some_function(&input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_feature_edge_case() {
        // Test boundary conditions, error cases, etc.
    }
}
```

**Test coverage expectations:**
- Core logic: 90%+
- API handlers: 80%+
- Database code: 85%+
- Overall: 80%+

```bash
# Check coverage
cargo tarpaulin --out Html --output-dir target/coverage
# Opens target/coverage/index.html
```

### 6. Create Pull Request

```bash
# Push branch to GitHub
git push origin feat/your-feature-name

# Create PR via GitHub web UI
# https://github.com/ruralpeds/Decision-trees-rust/compare
```

**PR requirements:**
- [ ] Tests added/updated
- [ ] Code formatted (cargo fmt)
- [ ] Linting passed (cargo clippy)
- [ ] Security audit passed (cargo audit)
- [ ] Documentation updated (if needed)
- [ ] Commit messages are clear

**PR description should include:**
```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How to test this change

## Checklist
- [ ] Tests added
- [ ] Documentation updated
- [ ] Breaking change (requires version bump)
```

### 7. Code Review

- Maintainers review PR within 48 hours
- May request changes (be responsive)
- Once approved, maintainer merges

## Code Standards

### Rust Style

**Follow Rust API guidelines:**
- https://rust-lang.github.io/api-guidelines/

**Code organization:**

```rust
// 1. Imports (std, external, internal)
use std::collections::HashMap;
use serde::Deserialize;
use crate::error::Result;

// 2. Type definitions
pub struct MyType {
    field: String,
}

// 3. Implementations
impl MyType {
    pub fn new(field: String) -> Self {
        Self { field }
    }
}

// 4. Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new() { ... }
}
```

**Naming conventions:**

```rust
// Constants: SCREAMING_SNAKE_CASE
const MAX_DEPTH: usize = 100;

// Types: PascalCase
struct ClinicalDecisionTree { }
enum EdgeCondition { }

// Functions/methods: snake_case
fn calculate_bmi(weight: f64) -> f64 { }

// Variables: snake_case
let session_id = Uuid::new_v4();
```

**Error handling:**

```rust
// Use Result<T> for recoverable errors
fn get_tree(id: Uuid) -> Result<Tree> {
    TreeRepository::get_by_id(id)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?
        .ok_or(Error::TreeNotFound(id.to_string()))
}

// Panic only for truly unrecoverable errors
// Prefer returning Err for recoverable issues
```

**Documentation:**

```rust
/// Clinical decision tree model with validation
///
/// Trees consist of nodes connected by edges with conditions.
/// Each edge evaluates to true/false based on user input.
///
/// # Example
/// ```
/// let tree = ClinicalDecisionTree {
///     id: Uuid::new_v4(),
///     title: "Fever Assessment".to_string(),
///     ...
/// };
/// ```
pub struct ClinicalDecisionTree {
    // ...
}

/// Validates tree structure for completeness and consistency
///
/// Checks:
/// - No orphaned nodes
/// - No cycles
/// - All paths terminate
/// - Child references exist
pub fn validate_tree(tree: &ClinicalDecisionTree) -> ValidationResult { }
```

### SQL Standards

**SQLx compile-time verification:**

```rust
// SQL queries must have verified sqlx-data.json
sqlx::query_as::<_, TreeRow>(
    r#"
    SELECT id, title, description, root_node_id, version, status, 
           created_at, updated_at
    FROM trees
    WHERE id = $1
    "#
)
.bind(tree_id)
.fetch_optional(pool)
.await?
```

**Migration naming:**

```
migrations/20260429000000_initial_schema.sql
migrations/20260429000001_audit_log.sql
migrations/20260430000000_add_tree_authors.sql
                 ↓
          Timestamp (YYYYMMDDHHMMSS)
```

## Commit Message Guidelines

**Format:**

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Example:**

```
feat(core): Add TreeValidator with 14-point validation

- Check for orphaned nodes (unreachable)
- Detect cycles in edge graph
- Verify all paths terminate at outcomes
- Validate input/output type compatibility
- Check FHIR prefill paths exist

Closes #123
```

**Types:**
- `feat:` New feature
- `fix:` Bug fix
- `refactor:` Code refactoring
- `test:` Test additions
- `docs:` Documentation
- `perf:` Performance improvement
- `chore:` Dependencies, build, etc.

**Scope:**
- `core:` cds-tree-core
- `api:` cds-tree-api
- `storage:` cds-tree-storage
- `fhir:` cds-tree-fhir
- `audit:` cds-tree-audit
- `ci:` GitHub Actions
- `docker:` Docker/Compose

## Sprint Structure

### 8-Sprint Roadmap

| Sprint | Focus | Duration | Status |
|--------|-------|----------|--------|
| 1 | Core evaluator | 2 weeks | ✅ |
| 2 | REST API | 2 weeks | ✅ |
| 3 | PostgreSQL | 2 weeks | ✅ |
| 4 | Session traversal | 2 weeks | ✅ |
| 5 | CDS Hooks + FHIR | 2 weeks | ✅ |
| 6 | Audit + export | 2 weeks | ✅ |
| 7 | WASM + offline | 2 weeks | Planned |
| 8 | Load testing | 2 weeks | Planned |

### Contributing to Sprints

**Each sprint:**
1. Opens with GitHub Project board
2. Issues labeled with sprint + priority
3. Daily standups (async updates in Discord)
4. Weekly review (Friday)
5. Sprint retro (end of sprint)

**To contribute:**
1. Watch the Sprint 7 board
2. Pick issues labeled `sprint-7`
3. Follow workflow above
4. PRs merged daily during sprint

## Performance Considerations

**Target metrics (from Sprint 8):**
- Tree creation: <50ms
- Node answer: <80ms
- Session outcome: <15ms
- Concurrent sessions: 10,000+
- RPS: 1,000+

**When optimizing:**
1. Benchmark first (use criterion.rs)
2. Profile with flamegraph
3. Minimize allocations
4. Cache frequently used data
5. Use connection pooling

## Documentation

**Update documentation when:**
- Adding new API endpoints
- Changing data models
- Modifying CLI commands
- Updating deployment process

**Documentation files:**
- `README.md` — Quick start
- `ARCHITECTURE.md` — System design
- `CONTRIBUTING.md` — This file
- `PROJECT_CATALOG.md` — Function reference
- `.../docs/` — Additional guides

## Testing Strategy

### Test Pyramid

```
         /\
        /  \  Integration tests (10%)
       /____\
      /      \
     / Unit   \ Unit tests (70%)
    /_________ \
   /           \
  / Smoke tests \ Smoke tests (20%)
 /_______________\
```

### Test Types

**Unit tests** (70%)
```rust
#[test]
fn test_evaluator_compare_operator() {
    let cond = EdgeCondition::Compare { ... };
    let result = cond.evaluate(&value);
    assert!(result);
}
```

**Integration tests** (10%)
```bash
# Tests multiple components together
cargo test --test '*'
# Requires PostgreSQL running
```

**Smoke tests** (20%)
```bash
# Basic happy path tests
cargo test --lib --release
```

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p cds-tree-core

# Specific test
cargo test test_evaluator

# With output
cargo test -- --nocapture

# Single-threaded (for database tests)
cargo test -- --test-threads=1
```

## Debugging

### Common Issues

**Database connection refused:**
```bash
docker-compose ps
docker-compose up -d postgres
```

**Port already in use:**
```bash
# Kill process on port 3000
lsof -ti:3000 | xargs kill -9
```

**Tests failing locally:**
```bash
# Ensure database is clean
docker-compose down -v
docker-compose up -d
cargo test --test '*' -- --test-threads=1
```

### Debug Logging

```rust
// Enable debug logging
RUST_LOG=debug cargo run -p cds-tree-api

// Filter specific modules
RUST_LOG=cds_tree_api=debug,cds_tree_core=trace cargo run -p cds-tree-api
```

## Deploying Changes

**Merging to main triggers:**
1. GitHub Actions CI/CD pipeline
2. Tests run automatically
3. Docker image builds
4. Image pushed to registry (if configured)

**Manual deployment:**
```bash
docker build -t ruralpeds/decision-trees-rust:latest .
docker push ruralpeds/decision-trees-rust:latest
```

## Questions?

- **Discord:** [Rural Pediatrics Community](https://discord.gg/ruralpeds)
- **GitHub Discussions:** [Project Discussions](https://github.com/ruralpeds/Decision-trees-rust/discussions)
- **Issues:** [Report bugs](https://github.com/ruralpeds/Decision-trees-rust/issues)

---

**Thank you for contributing to equitable clinical decision support!**
