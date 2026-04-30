# Architecture — Clinical Decision Tree Engine

## System Overview

A distributed clinical decision support system built on Rust with PostgreSQL persistence, CDS Hooks v2.0 integration, and FHIR R4 compliance.

```
┌─────────────────────────────────────────────────────────────────┐
│                    EHR Systems (Epic, Cerner)                   │
│                                                                 │
│  ┌──────────────────┐    ┌──────────────────┐                 │
│  │ Clinical UI      │    │ Patient Context  │                 │
│  │ (Tree authoring) │    │ (FHIR + SMART)   │                 │
│  └────────┬─────────┘    └────────┬─────────┘                 │
│           │                       │                            │
│           └───────────────────────┼────────────────────────────┘
│                                   │
│      REST API / CDS Hooks v2.0    │ FHIR R4
│                                   │
│        ┌──────────────────────────▼─────────────────────┐
│        │   cds-tree-api (Axum/Tokio)                    │
│        │                                                │
│        │  ┌─────────────────────────────────────────┐  │
│        │  │ Handlers:                               │  │
│        │  │ - Trees (CRUD)                         │  │
│        │  │ - Sessions (traversal)                 │  │
│        │  │ - CDS Hooks (discovery, services)      │  │
│        │  │ - Audit & Export (FHIR)               │  │
│        │  └─────────────────────────────────────────┘  │
│        │                                                │
│        │  ┌─────────────────────────────────────────┐  │
│        │  │ Middleware Stack:                       │  │
│        │  │ - Auth (JWT RS256)                     │  │
│        │  │ - Tracing (structured logging)         │  │
│        │  │ - Compression (gzip)                   │  │
│        │  │ - CORS (EHR origins)                   │  │
│        │  │ - Request ID (correlation)             │  │
│        │  └─────────────────────────────────────────┘  │
│        └──────────────────┬───────────────────────────┘
│                           │
│     ┌─────────────────────┼─────────────────────┐
│     │                     │                     │
│     ▼                     ▼                     ▼
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  │ cds-tree-    │  │ cds-tree-    │  │ cds-tree-    │
│  │ core         │  │ storage      │  │ fhir         │
│  │              │  │              │  │              │
│  │ • Tree model │  │ • PostgreSQL │  │ • FHIR R4    │
│  │ • Evaluator  │  │ • Repositories│  │ • CDS Hooks  │
│  │ • Validator  │  │ • Migrations │  │ • Adapters   │
│  │ • Calculators│  │ • Audit log  │  │ • Prefill    │
│  └──────────────┘  └──────────────┘  └──────────────┘
│        │                  │                    │
└────────┼──────────────────┼────────────────────┘
         │                  │
         └──────────────────┤
                            │
      ┌─────────────────────▼──────────────────┐
      │      PostgreSQL 16                     │
      │                                        │
      │  ┌──────────────────────────────────┐  │
      │  │ Tables:                          │  │
      │  │ - trees                          │  │
      │  │ - nodes                          │  │
      │  │ - sessions                       │  │
      │  │ - audit_log (partitioned)        │  │
      │  │ - tree_authors                   │  │
      │  │ - guideline_refs                 │  │
      │  └──────────────────────────────────┘  │
      └────────────────────────────────────────┘
```

## Crate Architecture

### 1. cds-tree-core (1,500 lines)

**Purpose:** Core decision tree model and evaluator. No I/O, no_std compatible.

**Modules:**

```
cds-tree-core/
├── src/
│   ├── error.rs
│   │   ├── EvalError               // Evaluation failed
│   │   ├── ValidationError         // Tree invalid
│   │   └── impl From<...> Error
│   │
│   ├── model/
│   │   ├── mod.rs
│   │   ├── tree.rs
│   │   │   ├── ClinicalDecisionTree
│   │   │   ├── TreeStatus (draft, peer_review, published, deprecated)
│   │   │   ├── EvidenceLevel (expert_opinion, consensus, ...research)
│   │   │   ├── MedicalSpecialty (Pediatrics, Oncology, etc.)
│   │   │   ├── ClinicalSetting (outpatient, emergency, hospital)
│   │   │   └── TreeAuthor
│   │   │
│   │   ├── node.rs
│   │   │   ├── DecisionNode
│   │   │   ├── NodeKind (Decision, Action, Outcome)
│   │   │   ├── ChildEdge
│   │   │   ├── EdgeCondition (Always, Compare, InRange, Contains, And, Or, Not)
│   │   │   ├── ComparisonOperator (Eq, Lt, Gt, Lte, Gte, Neq)
│   │   │   ├── ConditionVariable (input reference)
│   │   │   └── SkipCondition
│   │   │
│   │   ├── input.rs
│   │   │   ├── NodeInput
│   │   │   │   ├── Boolean { label, required }
│   │   │   │   ├── SingleSelect { label, options, required }
│   │   │   │   ├── MultiSelect { label, options, required }
│   │   │   │   ├── Slider { label, min, max, step }
│   │   │   │   ├── Numeric { label, min, max, unit }
│   │   │   │   ├── Computed { formula }
│   │   │   │   └── Display { value }
│   │   │   ├── SelectOption { code, label, snomed, loinc }
│   │   │   ├── ReferenceRange { lower, upper, unit }
│   │   │   ├── UnitConversion { from, to, factor }
│   │   │   ├── FhirPrefillMapping { loinc_code, snomed_code, fhir_path }
│   │   │   ├── ComputedFormula { expression, variables }
│   │   │   └── BuiltInFormula (BMI, CorrectedGA, CrCl, MAP, eGFR, PediatricDose)
│   │   │
│   │   ├── outcome.rs
│   │   │   ├── OutcomePayload
│   │   │   ├── SeverityLevel (Info, Suggestion, Warning, Critical)
│   │   │   ├── RecommendedAction
│   │   │   ├── ActionType (create, update, delete, append)
│   │   │   ├── PathSummaryStep
│   │   │   └── GuidelineRef
│   │   │
│   │   └── evidence.rs
│   │       ├── EvidenceRef
│   │       ├── EvidenceType (guideline, literature, consensus)
│   │       └── GuidelineRef
│   │
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── evaluator.rs
│   │   │   ├── Evaluator { tree: &ClinicalDecisionTree }
│   │   │   ├── evaluate(node_id, inputs) -> Result<(next_node_id, path)>
│   │   │   ├── evaluate_edge(condition, inputs) -> Result<bool>
│   │   │   ├── evaluate_computed(formula) -> Result<Value>
│   │   │   ├── SessionState = HashMap<Uuid, ConditionValue>
│   │   │   └── Supports: recursive descent, short-circuit evaluation
│   │   │
│   │   └── validator.rs
│   │       ├── TreeValidator
│   │       ├── validate(tree) -> ValidationReport
│   │       ├── Checks:
│   │       │  1. No orphaned nodes
│   │       │  2. No cycles in edge graph
│   │       │  3. All paths terminate at outcomes
│   │       │  4. Root node exists and is reachable
│   │       │  5. All child references exist
│   │       │  6. No unreachable outcome nodes
│   │       │  7. Input types match edge conditions
│   │       │  8. No duplicate node IDs
│   │       │  9. All referenced calculations are valid
│   │       │ 10. Evidence references are valid
│   │       │ 11. FHIR prefill paths are syntactically valid
│   │       │ 12. Coded values (SNOMED/LOINC) are valid
│   │       │ 13. Weights sum correctly if present
│   │       │ 14. No conflicting clinical decisions
│   │       └── Returns: ValidationReport { errors, warnings, suggestions }
│   │
│   └── lib.rs (re-exports)
│
└── tests/
    ├── evaluator_tests.rs
    ├── validator_tests.rs
    └── calculator_tests.rs
```

**Key Types:**

```rust
pub struct ClinicalDecisionTree {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub root_node_id: Uuid,
    pub version: String,
    pub status: TreeStatus,
    pub evidence_level: EvidenceLevel,
    pub specialty: MedicalSpecialty,
    pub setting: ClinicalSetting,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authors: Vec<TreeAuthor>,
    pub guidelines: Vec<GuidelineRef>,
}

pub struct DecisionNode {
    pub id: Uuid,
    pub tree_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub label: String,
    pub description: Option<String>,
    pub kind: NodeKind,
    pub input: Option<NodeInput>,
    pub children: Vec<ChildEdge>,
    pub outcome: Option<OutcomePayload>,
    pub depth: i32,
}

pub enum EdgeCondition {
    Always,                                    // True always
    Compare { var: ConditionVariable, op: ComparisonOperator, value: ConditionValue },
    InRange { var: ConditionVariable, lower: f64, upper: f64 },
    Contains { var: ConditionVariable, substring: String },
    And(Box<EdgeCondition>, Box<EdgeCondition>),
    Or(Box<EdgeCondition>, Box<EdgeCondition>),
    Not(Box<EdgeCondition>),
}

pub enum BuiltInFormula {
    BMI { weight_kg: f64, height_cm: f64 },
    CorrectedGA { chrono_age: f64, birth_ga: f64 },
    CrCl { age: f64, weight: f64, creatinine: f64, is_male: bool },
    MAP { systolic: f64, diastolic: f64 },
    eGFR { creatinine: f64, age: f64, female: bool },
    PediatricDose { weight_kg: f64, dose_per_kg: f64 },
}
```

**No Dependencies (core):**
- No async, no networking, no I/O
- Pure computation only
- ~300KB WASM binary when compiled to WebAssembly

### 2. cds-tree-api (1,200 lines)

**Purpose:** HTTP REST API server with middleware stack.

**Architecture:**

```
cds-tree-api/
├── src/
│   ├── main.rs
│   │   ├── Init tracing + metrics
│   │   ├── Load .env config
│   │   ├── Connect to PostgreSQL
│   │   ├── Run migrations (if AUTO_MIGRATE=true)
│   │   ├── Build router
│   │   ├── Start Tokio server on 0.0.0.0:3000
│   │   └── Handle graceful shutdown
│   │
│   ├── config.rs
│   │   ├── Config struct (from env)
│   │   │   ├── DATABASE_URL
│   │   │   ├── LOG_LEVEL
│   │   │   ├── JWT_SECRET
│   │   │   ├── ALLOWED_ORIGINS (CORS)
│   │   │   └── SERVER_PORT
│   │   └── JwtClaims struct
│   │       ├── sub (subject/user ID)
│   │       ├── roles (Vec<String>)
│   │       ├── iat (issued at)
│   │       └── exp (expiration)
│   │
│   ├── state.rs
│   │   ├── AppState (wrapped in Arc)
│   │   │   ├── config: Arc<Config>
│   │   │   ├── db_pool: PgPool (sqlx)
│   │   │   └── metrics: Arc<Registry> (prometheus)
│   │   └── impl Clone for AppState
│   │
│   ├── error.rs
│   │   ├── AppError enum
│   │   │   ├── TreeNotFound(String)
│   │   │   ├── NodeNotFound(String)
│   │   │   ├── SessionNotFound(String)
│   │   │   ├── DatabaseError(String)
│   │   │   ├── ValidationError(String)
│   │   │   ├── InvalidInput(String)
│   │   │   ├── Unauthorized
│   │   │   ├── Forbidden
│   │   │   └── InternalError
│   │   ├── impl IntoResponse for AppError
│   │   │   └── Returns JSON error response
│   │   └── ApiResult<T> = Result<T, AppError>
│   │
│   ├── router.rs
│   │   ├── build_router(state: AppState) -> Router
│   │   └── Nested routes:
│   │       ├── /healthz -> health_check
│   │       ├── /api/v1/health -> {/, /live, /ready}
│   │       ├── /api/v1/{trees, nodes, sessions, audit}
│   │       ├── /cds-services -> {discovery, services, feedback}
│   │       └── /metrics -> prometheus metrics
│   │
│   ├── middleware/
│   │   ├── auth.rs
│   │   │   ├── JwtClaims extractor (FromRequestParts)
│   │   │   ├── extract_token_from_header()
│   │   │   ├── verify_jwt()
│   │   │   └── extract_roles()
│   │   │
│   │   └── tracing.rs
│   │       ├── init_tracing() -> TraceLayer + JSON output
│   │       ├── init_json_tracing()
│   │       └── Spans for each request
│   │
│   ├── handlers/
│   │   ├── mod.rs (re-exports all handler functions)
│   │   │
│   │   ├── health.rs
│   │   │   ├── GET /health -> { status: "healthy" }
│   │   │   ├── GET /health/live -> Kubernetes liveness
│   │   │   └── GET /health/ready -> Readiness (DB check)
│   │   │
│   │   ├── metrics.rs
│   │   │   └── GET /metrics -> Prometheus exposition format
│   │   │
│   │   ├── trees.rs
│   │   │   ├── POST /trees -> create_tree()
│   │   │   ├── GET /trees -> list_trees()
│   │   │   ├── GET /trees/:id -> get_tree()
│   │   │   ├── PUT /trees/:id -> update_tree()
│   │   │   ├── DELETE /trees/:id -> delete_tree() [soft]
│   │   │   ├── POST /trees/:id/publish -> publish_tree()
│   │   │   └── POST /trees/:id/validate -> validate_tree()
│   │   │
│   │   ├── nodes.rs
│   │   │   ├── POST /trees/:tree_id/nodes -> create_node()
│   │   │   ├── GET /trees/:tree_id/nodes -> list_nodes()
│   │   │   ├── GET /trees/:tree_id/nodes/:id -> get_node()
│   │   │   ├── PUT /trees/:tree_id/nodes/:id -> update_node()
│   │   │   ├── DELETE /trees/:tree_id/nodes/:id -> delete_node() [cascade]
│   │   │   └── GET /trees/:tree_id/nodes/:id/children -> get_children()
│   │   │
│   │   ├── sessions.rs
│   │   │   ├── POST /sessions -> create_session()
│   │   │   │   ├── Fetch tree (must be published)
│   │   │   │   ├── Create session row
│   │   │   │   ├── Record audit: session_start
│   │   │   │   └── Return root node
│   │   │   │
│   │   │   ├── GET /sessions/:id -> get_current_node()
│   │   │   │   ├── Get session + current node
│   │   │   │   ├── Calculate progress
│   │   │   │   └── Return node + breadcrumb
│   │   │   │
│   │   │   ├── POST /sessions/:id/advance -> advance_session()
│   │   │   │   ├── Validate node exists + session owns it
│   │   │   │   ├── Record answer in session
│   │   │   │   ├── Evaluate edge conditions (SessionStateMachine)
│   │   │   │   ├── Get next node
│   │   │   │   ├── Record audit: node_answer
│   │   │   │   ├── If outcome: record session_completed
│   │   │   │   └── Return next node
│   │   │   │
│   │   │   ├── GET /sessions/:id/path -> get_session_path()
│   │   │   │   └── Return breadcrumb (node labels + answers)
│   │   │   │
│   │   │   ├── GET /sessions/:id/outcome -> get_session_outcome()
│   │   │   │   └── Return outcome payload
│   │   │   │
│   │   │   └── POST /sessions/:id/abandon -> abandon_session()
│   │   │       ├── Mark session as abandoned
│   │   │       └── Record audit: session_abandoned
│   │   │
│   │   ├── hooks.rs (CDS Hooks v2.0)
│   │   │   ├── GET /cds-services -> cds_services_discovery()
│   │   │   ├── GET /cds-services/metadata -> cds_service_metadata()
│   │   │   ├── POST /cds-services/fever-assessment -> fever_assessment_service()
│   │   │   ├── POST /cds-services/antibiotic-stewardship -> antibiotic_stewardship_service()
│   │   │   ├── POST /cds-services/order-safety-review -> order_safety_review_service()
│   │   │   └── POST /cds-services/:service-id/feedback -> record_cds_feedback()
│   │   │
│   │   ├── audit.rs
│   │   │   ├── GET /sessions/:id/audit-log -> get_session_audit_log()
│   │   │   ├── GET /sessions/:id/fhir-audit-event -> export_session_as_fhir_audit_event()
│   │   │   ├── GET /sessions/:id/fhir-observations -> export_session_as_fhir_observations()
│   │   │   ├── GET /sessions/:id/summary -> get_session_summary()
│   │   │   ├── GET /clinicians/:id/audit-log -> get_clinician_audit_log()
│   │   │   └── GET /patients/:id/audit-log -> get_patient_audit_log()
│   │   │
│   │   └── traversal.rs (state machine)
│   │       ├── SessionStateMachine
│   │       ├── new(tree, core_state)
│   │       ├── current_node()
│   │       ├── advance(answer) -> next_node_id
│   │       ├── root_node()
│   │       ├── path() -> Vec<BreadcrumbStep>
│   │       ├── is_terminal()
│   │       └── estimate_remaining_nodes()
│   │
│   └── lib.rs
│
└── tests/
    └── integration_tests.rs (requires PostgreSQL)
```

**Middleware Stack (order matters):**

```
Request
  │
  ├─ CORS (allow EHR origins)
  ├─ Request ID (UUID, added to response header)
  ├─ Tracing (begin span, structured logging)
  ├─ Compression (gzip if Accept-Encoding: gzip)
  ├─ Auth (JWT extraction + validation, optional)
  └─ Handler
       │
       └─ Response
```

### 3. cds-tree-storage (2,050 lines)

**Purpose:** PostgreSQL persistence layer using SQLx.

```
cds-tree-storage/
├── src/
│   ├── models.rs
│   │   ├── Database row types:
│   │   │   ├── TreeRow { id, title, root_node_id, version, status, ... }
│   │   │   ├── NodeRow { id, tree_id, parent_id, label, kind, depth, ... }
│   │   │   ├── SessionRow { id, tree_id, clinician_id, patient_id, status, ... }
│   │   │   └── AuditLogRow { id, session_id, event_type, event_data, occurred_at, ... }
│   │   │
│   │   └── Request/response DTOs:
│   │       ├── CreateTreeRequest
│   │       ├── UpdateTreeRequest
│   │       ├── TreeResponse
│   │       ├── CreateNodeRequest
│   │       ├── NodeResponseForSession
│   │       ├── CreateSessionRequest
│   │       ├── AdvanceSessionRequest
│   │       ├── SessionResponse
│   │       └── ValidationReportResponse
│   │
│   ├── error.rs
│   │   ├── StorageError enum
│   │   ├── impl From<sqlx::Error>
│   │   └── StorageResult<T>
│   │
│   ├── lib.rs
│   │   ├── run_migrations(pool) -> Result<()>
│   │   └── Re-exports
│   │
│   └── repo/
│       ├── mod.rs
│       │   └── Re-exports all repositories
│       │
│       ├── trees.rs
│       │   ├── TreeRepository (all methods are static/async)
│       │   ├── create(pool, req) -> TreeRow
│       │   ├── get_by_id(pool, id) -> Option<TreeRow>
│       │   ├── get_by_slug(pool, slug) -> Option<TreeRow>
│       │   ├── list(pool, limit, offset) -> Vec<TreeRow>
│       │   ├── list_by_status(pool, status) -> Vec<TreeRow>
│       │   ├── update(pool, id, req) -> TreeRow
│       │   ├── soft_delete(pool, id)
│       │   ├── publish(pool, id) -> TreeRow
│       │   ├── clone(pool, id) -> TreeRow
│       │   ├── set_root_node(pool, id, node_id)
│       │   └── count(pool) -> i64
│       │
│       ├── nodes.rs
│       │   ├── NodeRepository
│       │   ├── create(pool, tree_id, req) -> NodeRow
│       │   ├── get_by_id(pool, id) -> Option<NodeRow>
│       │   ├── list_by_tree(pool, tree_id) -> Vec<NodeRow>
│       │   ├── get_children(pool, id) -> Vec<(NodeRow, ChildEdge)>
│       │   ├── get_root(pool, tree_id) -> NodeRow
│       │   ├── update(pool, id, req) -> NodeRow
│       │   ├── set_children(pool, id, children)
│       │   ├── set_outcome(pool, id, outcome)
│       │   ├── delete_cascade(pool, id) [deletes children recursively]
│       │   └── count_by_tree(pool, tree_id) -> i64
│       │
│       ├── sessions.rs
│       │   ├── SessionRepository
│       │   ├── create(pool, tree_id, version, ...) -> SessionRow
│       │   ├── get_by_id(pool, id) -> Option<SessionRow>
│       │   ├── record_answer(pool, session_id, node_id, answer)
│       │   ├── get_answers(pool, session_id) -> HashMap<Uuid, Value>
│       │   ├── set_current_node(pool, session_id, node_id)
│       │   ├── complete(pool, session_id, outcome_node_id)
│       │   ├── abandon(pool, session_id, reason)
│       │   ├── list_by_clinician(pool, clinician_id) -> Vec<SessionRow>
│       │   └── count_active(pool) -> i64
│       │
│       └── audit.rs (Sprint 6)
│           ├── AuditLogRepository
│           ├── record_session_start(pool, ...)
│           ├── record_node_answer(pool, ...)
│           ├── record_session_completed(pool, ...)
│           ├── record_session_abandoned(pool, ...)
│           ├── record_cds_recommendation(pool, ...)
│           ├── get_session_events(pool, session_id) -> Vec<AuditLogRow>
│           ├── get_clinician_events(pool, clinician_id, limit, offset)
│           ├── get_patient_events(pool, patient_id, limit, offset)
│           ├── get_tree_events(pool, tree_id, limit, offset)
│           └── count_event_type(pool, session_id, type)
│
├── migrations/
│   ├── 20260429000000_initial_schema.sql
│   │   ├── CREATE TABLE trees
│   │   ├── CREATE TABLE nodes
│   │   ├── CREATE TABLE sessions
│   │   ├── CREATE TABLE tree_authors
│   │   └── CREATE TABLE guideline_refs
│   │
│   ├── 20260429000001_audit_log.sql
│   │   ├── CREATE TABLE audit_log
│   │   ├── Partitioning by month
│   │   └── Insert-only rules (no UPDATE/DELETE)
│   │
│   └── sqlx-data.json (compile-time SQL verification)
│
└── tests/
    └── integration_tests.rs (requires PostgreSQL)
```

### 4. cds-tree-fhir (1,280 lines)

**Purpose:** FHIR R4 and CDS Hooks v2.0 integration.

```
cds-tree-fhir/
├── src/
│   ├── models.rs
│   │   ├── CDS Hooks v2.0
│   │   │   ├── CdsHooksRequest { hook, hook_instance, context, prefetch }
│   │   │   ├── CdsHooksResponse { cards, errors }
│   │   │   ├── CdsCard { uuid, summary, indicator, detail, suggestions, links }
│   │   │   ├── CdsSuggestion { uuid, label, actions }
│   │   │   ├── CdsAction { action_type, description, resource }
│   │   │   ├── CdsLink { label, url, open_in_new_tab }
│   │   │   ├── DiscoveryResponse { services }
│   │   │   └── CdsService { id, hook, title, description, prefetch }
│   │   │
│   │   ├── FHIR R4 (simplified)
│   │   │   ├── FhirPatient { id, birth_date, gender, name }
│   │   │   ├── FhirObservation { id, code, value_quantity, value_code, effective_datetime }
│   │   │   ├── FhirQuantity { value, unit, code }
│   │   │   ├── FhirCodeableConcept { coding, text }
│   │   │   ├── FhirCoding { system, code, display }
│   │   │   ├── FhirEncounter { id, status, period }
│   │   │   ├── FhirAuditEvent { id, type, action, period, recorded, outcome, detail }
│   │   │   └── FhirPeriod { start, end }
│   │   │
│   │   └── SMART Launch
│   │       ├── SmartLaunchRequest { launch, iss }
│   │       ├── SmartTokenResponse { access_token, patient, encounter, provider }
│   │       └── CapabilityStatement { authorization_endpoint, token_endpoint, ... }
│   │
│   ├── adapters/
│   │   ├── prefill.rs
│   │   │   ├── FhirPrefillAdapter
│   │   │   │   ├── extract_observation_value(obs, type) -> Value
│   │   │   │   ├── extract_weight_kg(obs) -> f64 [lbs→kg]
│   │   │   │   ├── extract_temperature_celsius(obs) -> f64 [F→C]
│   │   │   │   ├── extract_gestational_age_weeks(obs) -> f64 [days→weeks]
│   │   │   │   ├── extract_blood_pressure(obs) -> (systolic, diastolic)
│   │   │   │   ├── extract_respiratory_rate(obs) -> f64
│   │   │   │   ├── matches_loinc_code(obs, code) -> bool
│   │   │   │   ├── matches_snomed_code(obs, code) -> bool
│   │   │   │   ├── get_loinc_code(obs) -> Option<String>
│   │   │   │   └── format_for_display(obs) -> String
│   │   │   │
│   │   │   └── ClinicalCalculators
│   │   │       ├── calculate_bmi(weight, height) -> f64
│   │   │       ├── calculate_corrected_ga(chrono, birth) -> f64
│   │   │       ├── calculate_crcl(age, weight, creatinine, male) -> f64
│   │   │       ├── calculate_map(systolic, diastolic) -> f64
│   │   │       ├── calculate_pediatric_dose(weight, dose_per_kg) -> f64
│   │   │       ├── classify_fever_severity(temp) -> String
│   │   │       └── classify_respiratory_distress(rr, age) -> String
│   │   │
│   │   └── export.rs (Sprint 6)
│   │       ├── FhirExportAdapter
│   │       ├── session_to_audit_event(...) -> FhirAuditEvent
│   │       ├── node_answer_to_observation(...) -> FhirObservation
│   │       ├── build_session_summary(...) -> Value
│   │       └── generate_decision_justification(...) -> String
│   │
│   ├── hooks/
│   │   ├── service.rs
│   │   │   ├── CdsHooksService
│   │   │   │   ├── outcome_to_card(outcome, session_id, tree_id) -> CdsCard
│   │   │   │   ├── create_response(cards) -> CdsHooksResponse
│   │   │   │   ├── create_error_response(errors) -> CdsHooksResponse
│   │   │   │   └── build_discovery(...) -> CdsService
│   │   │   │
│   │   │   └── HookType enum
│   │   │       ├── PatientView
│   │   │       ├── MedicationOrder
│   │   │       ├── OrderReview
│   │   │       ├── OrderSign
│   │   │       ├── as_str() -> &str
│   │   │       ├── description() -> &str
│   │   │       └── prefetch_resources() -> Vec<(key, fhir_path)>
│   │   │
│   │   └── mod.rs
│   │
│   └── lib.rs
│       └── Re-exports all public items
│
└── tests/
    └── Unit tests for adapters and models
```

### 5. Database Schema

```sql
-- trees table
CREATE TABLE trees (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    root_node_id UUID,
    version VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL, -- draft, peer_review, published, deprecated
    evidence_level VARCHAR(100),  -- expert_opinion, consensus, ...research
    specialty VARCHAR(100),        -- Pediatrics, Oncology, ...
    clinical_setting VARCHAR(100), -- outpatient, emergency, hospital
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,          -- soft delete
    UNIQUE(title, version)
);
CREATE INDEX idx_trees_status ON trees(status);
CREATE INDEX idx_trees_deleted_at ON trees(deleted_at);

-- nodes table
CREATE TABLE nodes (
    id UUID PRIMARY KEY,
    tree_id UUID NOT NULL REFERENCES trees(id),
    parent_id UUID REFERENCES nodes(id),
    label VARCHAR(255) NOT NULL,
    description TEXT,
    kind VARCHAR(50) NOT NULL, -- Decision, Action, Outcome
    input JSONB,               -- NodeInput serialized
    children JSONB,            -- Vec<ChildEdge> serialized
    outcome JSONB,             -- OutcomePayload serialized
    depth INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    CONSTRAINT depth_valid CHECK (depth >= 0 AND depth <= 100)
);
CREATE INDEX idx_nodes_tree_id ON nodes(tree_id);
CREATE INDEX idx_nodes_parent_id ON nodes(parent_id);
CREATE INDEX idx_nodes_deleted_at ON nodes(deleted_at);

-- sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    tree_id UUID NOT NULL REFERENCES trees(id),
    tree_version VARCHAR(50) NOT NULL,
    clinician_id VARCHAR(255),
    patient_id VARCHAR(255),
    current_node_id UUID REFERENCES nodes(id),
    outcome_node_id UUID REFERENCES nodes(id),
    status VARCHAR(50) NOT NULL DEFAULT 'active', -- active, completed, abandoned
    answers JSONB DEFAULT '{}',  -- { node_id: value, ... }
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    abandoned_at TIMESTAMP,
    context VARCHAR(100)         -- UI, CDS_Hooks, etc.
);
CREATE INDEX idx_sessions_tree_id ON sessions(tree_id);
CREATE INDEX idx_sessions_clinician_id ON sessions(clinician_id);
CREATE INDEX idx_sessions_patient_id ON sessions(patient_id);
CREATE INDEX idx_sessions_status ON sessions(status);

-- audit_log table (partitioned by month, insert-only)
CREATE TABLE audit_log (
    id BIGSERIAL,
    session_id UUID NOT NULL,
    tree_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB NOT NULL,
    occurred_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    clinician_id VARCHAR(255),
    patient_id VARCHAR(255),
    PRIMARY KEY (id, occurred_at)
) PARTITION BY RANGE (occurred_at);

-- Create monthly partitions
CREATE TABLE audit_log_2026_01 PARTITION OF audit_log
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE audit_log_2026_02 PARTITION OF audit_log
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
-- ... etc

CREATE INDEX idx_audit_log_session ON audit_log(session_id);
CREATE INDEX idx_audit_log_tree ON audit_log(tree_id);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX idx_audit_log_occurred ON audit_log(occurred_at DESC);
```

## Request/Response Patterns

### Tree Creation Flow

```
POST /api/v1/trees
{
  "title": "Fever Assessment",
  "description": "Pediatric fever evaluation",
  "specialty": "Pediatrics",
  "clinical_setting": "outpatient"
}

→ 200 OK
{
  "id": "tree-uuid",
  "title": "Fever Assessment",
  "root_node_id": null,     // Set when first node created
  "status": "draft",
  "version": "1.0.0",
  "created_at": "2026-04-29T12:00:00Z"
}
```

### Session Traversal Flow

```
POST /api/v1/sessions
{ "tree_id": "tree-uuid", "patient_id": "pt-123" }

→ 200 OK
{
  "session_id": "sess-uuid",
  "current_node_id": "root-node-uuid",
  "node": {
    "id": "root-node-uuid",
    "label": "Is patient febrile (>38.0°C)?",
    "kind": "Decision",
    "input": { "type": "Boolean" }
  },
  "progress": {
    "current_depth": 0,
    "estimated_remaining": 3,
    "breadcrumb": []
  }
}

---

POST /api/v1/sessions/sess-uuid/advance
{ "answer": true }

→ 200 OK
{
  "current_node_id": "next-node-uuid",
  "node": {
    "id": "next-node-uuid",
    "label": "How long has fever lasted (hours)?",
    "kind": "Decision",
    "input": { "type": "Numeric", "min": 0, "max": 1440 }
  },
  "progress": {
    "current_depth": 1,
    "estimated_remaining": 2,
    "breadcrumb": [
      { "node_id": "root", "label": "Is febrile?", "answer": true, "depth": 0 }
    ]
  }
}

---

GET /api/v1/sessions/sess-uuid/outcome

→ 200 OK
{
  "outcome_node_id": "outcome-uuid",
  "title": "Mild Fever",
  "severity": "warning",
  "summary": "Patient has mild fever with short duration",
  "recommendation": "Monitor vital signs, provide supportive care",
  "recommended_actions": [
    {
      "title": "Monitor temperature",
      "description": "Check fever every 4 hours",
      "action_type": "create"
    }
  ],
  "icd10_codes": ["R50.9"],
  "snomed_codes": ["386661006"]
}
```

## Performance & Scalability

**Benchmarks (target):**
- Tree creation: <50ms (DB insert)
- Node answer: <80ms (edge evaluation + DB updates)
- Session outcome: <15ms (single DB fetch)
- FHIR export: <25ms (JSON serialization)
- Concurrent: 10,000+ active sessions
- RPS: 1,000+ (tested in Sprint 8)

**Optimization strategies:**
1. Connection pooling (sqlx::Pool)
2. Compiled SQL queries (SQLx)
3. Async/await throughout (Tokio)
4. JSONB indexes (PostgreSQL)
5. Partitioned audit log (no single table bloat)
6. Stateless API (scales horizontally)

## Security

**Authentication:** JWT RS256
- Issued by identity provider (Keycloak, Auth0, etc.)
- Verified by API middleware
- Claims: sub (user ID), roles, exp (expiration)

**Authorization:** Role-based access control
- `tree:read` - Read trees
- `tree:write` - Create/edit trees
- `tree:publish` - Publish trees
- `session:create` - Start assessments
- `audit:read` - View audit trails

**Database security:**
- Audit log insert-only (no tampering)
- Soft deletes preserve history
- Encrypted passwords in database (if storing)
- All queries parameterized (SQLi prevention)

**Network:**
- HTTPS/TLS in production
- CORS restricted to EHR origins
- Rate limiting (TODO: Sprint 7)
- Request ID correlation

## Compliance

**HIPAA:**
- Audit trail captures who, what, when
- Clinician + patient tracking
- De-identification support (TODO)

**GDPR:**
- Right-to-access (patient audit trail)
- Right-to-deletion (soft delete with cleanup)
- Data portability (FHIR export)

**FDA 21 CFR Part 11:**
- Immutable audit trail (insert-only)
- Unique IDs (UUID)
- User identification (clinician_id)
- Timestamps (RFC3339)

---

**All systems documented in production code with `///` doc comments.**

See PROJECT_CATALOG.md for complete function reference.
