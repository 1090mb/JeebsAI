-- CDHSC Proposals: Community-driven brain updates with approval workflow
CREATE TABLE IF NOT EXISTS cdhsc_proposals (
    id TEXT PRIMARY KEY NOT NULL,
    proposal_type TEXT NOT NULL DEFAULT 'add_node',  -- add_node, remove_node, link_nodes, update_metadata
    status TEXT NOT NULL DEFAULT 'pending',          -- pending, approved, denied, implemented
    title TEXT NOT NULL,
    description TEXT,
    proposer_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    -- Node/link data
    node_id TEXT,
    fact TEXT,
    source TEXT,
    from_node_id TEXT,
    to_node_id TEXT,
    relation_type TEXT,

    -- Voting/approval
    votes_for INTEGER DEFAULT 0,
    votes_against INTEGER DEFAULT 0,
    approved_by TEXT,
    approved_at TEXT,
    denied_reason TEXT,

    FOREIGN KEY (proposer_id) REFERENCES users(username)
);

CREATE INDEX IF NOT EXISTS idx_cdhsc_proposals_status ON cdhsc_proposals(status);
CREATE INDEX IF NOT EXISTS idx_cdhsc_proposals_proposer ON cdhsc_proposals(proposer_id);
CREATE INDEX IF NOT EXISTS idx_cdhsc_proposals_created ON cdhsc_proposals(created_at);
CREATE INDEX IF NOT EXISTS idx_cdhsc_proposals_type ON cdhsc_proposals(proposal_type);

-- Votes on proposals
CREATE TABLE IF NOT EXISTS cdhsc_proposal_votes (
    id TEXT PRIMARY KEY NOT NULL,
    proposal_id TEXT NOT NULL,
    voter_id TEXT NOT NULL,
    vote_type TEXT NOT NULL,  -- approve, deny
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (proposal_id) REFERENCES cdhsc_proposals(id),
    FOREIGN KEY (voter_id) REFERENCES users(username),
    UNIQUE(proposal_id, voter_id)
);

CREATE INDEX IF NOT EXISTS idx_cdhsc_proposal_votes_proposal ON cdhsc_proposal_votes(proposal_id);
CREATE INDEX IF NOT EXISTS idx_cdhsc_proposal_votes_voter ON cdhsc_proposal_votes(voter_id);
