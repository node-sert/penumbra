-- Application state, stored in single-row tables
CREATE TABLE chain_params (bytes BLOB);
CREATE TABLE full_viewing_key (bytes BLOB);
CREATE TABLE sync_height (height BIGINT);
CREATE TABLE note_commitment_tree (bytes BLOB);

-- Minimal data required for balance tracking
CREATE TABLE notes (
    note_commitment         BLOB PRIMARY KEY NOT NULL,
    height_spent            BIGINT, --null if unspent, otherwise spent at height_spent 
    height_created          BIGINT NOT NULL,
    -- note contents themselves:
    diversifier             BLOB NOT NULL,
    amount                  BIGINT NOT NULL,
    asset_id                BLOB NOT NULL,
    transmission_key        BLOB NOT NULL,
    blinding_factor         BLOB NOT NULL, 
    -- precomputed decryption of the diversifier
    diversifier_index       BLOB NOT NULL,
    -- the nullifier for this note, used to detect when it is spent
    nullifier               BLOB NOT NULL
);

-- general purpose note queries
CREATE INDEX notes_idx ON notes (
    height_spent,       -- null if unspent, so spent/unspent is first
    diversifier_index,  -- then filter by account
    asset_id,           -- then by asset
    amount,             -- then by amount
    height_created      -- we don't really care about this, except informationally
);

-- used to detect spends
CREATE INDEX nullifier_idx on notes ( nullifier );
