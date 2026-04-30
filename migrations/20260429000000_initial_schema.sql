-- Initial schema for cds-tree-rs
-- Creates tables for trees, nodes, and sessions

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Trees table
CREATE TABLE IF NOT EXISTS trees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    version TEXT NOT NULL DEFAULT '0.1.0',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'peer_review', 'published', 'deprecated', 'archived')),
    evidence_level TEXT NOT NULL DEFAULT 'expert_consensus',
    root_node_id UUID,
    review_cycle_months SMALLINT NOT NULL DEFAULT 12,
    cds_hooks_id TEXT UNIQUE,
    fhir_library_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    last_reviewed_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_trees_slug ON trees(slug) WHERE deleted_at IS NULL;
CREATE INDEX idx_trees_status ON trees(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_trees_cds_hooks_id ON trees(cds_hooks_id) WHERE deleted_at IS NULL;

-- Nodes table (flat structure with parent references)
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tree_id UUID NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES nodes(id) ON DELETE CASCADE,
    depth INTEGER NOT NULL DEFAULT 0,
    node_kind TEXT NOT NULL DEFAULT 'question' CHECK (node_kind IN ('question', 'information', 'calculation', 'outcome', 'redirect')),
    label TEXT NOT NULL,
    description TEXT,
    -- NodeInput stored as JSONB
    input JSONB NOT NULL DEFAULT '{"type":"display"}'::jsonb,
    -- Outcome payload (only for outcome nodes)
    outcome JSONB,
    -- Child edges (array of ChildEdge)
    children JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- Evidence references (array of EvidenceRef)
    evidence_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    aria_label TEXT NOT NULL,
    required BOOLEAN NOT NULL DEFAULT TRUE,
    skip_condition JSONB,
    fhir_prefill JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_nodes_tree_id ON nodes(tree_id);
CREATE INDEX idx_nodes_parent_id ON nodes(parent_id);
CREATE INDEX idx_nodes_kind ON nodes(node_kind);
CREATE INDEX idx_nodes_input_type ON nodes USING GIN((input->'type'));

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tree_id UUID NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    tree_version TEXT NOT NULL,
    clinician_id TEXT,
    patient_id TEXT,
    fhir_encounter_id TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'abandoned')),
    current_node_id UUID REFERENCES nodes(id) ON DELETE SET NULL,
    fhir_context JSONB,
    -- Answers: map of node_id (UUID string) -> answer value
    answers JSONB NOT NULL DEFAULT '{}'::jsonb,
    source_context TEXT NOT NULL DEFAULT 'standalone',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    abandoned_at TIMESTAMPTZ,
    outcome_node_id UUID REFERENCES nodes(id) ON DELETE SET NULL
);

CREATE INDEX idx_sessions_tree_id ON sessions(tree_id);
CREATE INDEX idx_sessions_clinician_id ON sessions(clinician_id);
CREATE INDEX idx_sessions_patient_id ON sessions(patient_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_created ON sessions(started_at DESC);

-- Tree authors (junction table)
CREATE TABLE IF NOT EXISTS tree_authors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tree_id UUID NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    title TEXT,
    organization TEXT,
    email TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tree_authors_tree_id ON tree_authors(tree_id);

-- Guideline references
CREATE TABLE IF NOT EXISTS guideline_refs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tree_id UUID NOT NULL REFERENCES trees(id) ON DELETE CASCADE,
    organization TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT,
    publication_year SMALLINT,
    section TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_guideline_refs_tree_id ON guideline_refs(tree_id);
