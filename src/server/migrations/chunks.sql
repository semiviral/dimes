CREATE TABLE IF NOT EXISTS shards
(
    id UUID PRIMARY KEY,
    agent TEXT NOT NULL,
    max_chunks INT NOT NULL,
    chunks INT NOT NULL,
);

CREATE TABLE IF NOT EXISTS groupings
(
    id UUID PRIMARY KEY,
)

CREATE TABLE IF NOT EXISTS chunk_lookup
(
    hash BYTEA PRIMARY KEY,
    created DATETIME NOT NULL,
    grouping UUID REFERENCES groupings(id),
    seq INT NOT NULL,
);

CREATE TABLE IF NOT EXISTS chunk_placement
(
    shard_id UUID REFERENCES shard(id),
    chunk_hash BYTEA REFERNCES chunk_store(hash),
);