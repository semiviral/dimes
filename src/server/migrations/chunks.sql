CREATE TABLE IF NOT EXISTS shards
(
    id UUID PRIMARY KEY,
    agent TEXT NOT NULL,
    max_chunks BIGINT NOT NULL,
    chunks BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS groupings
(
    id UUID PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS chunk_lookup
(
    hash BYTEA PRIMARY KEY,
    created TIMESTAMP WITH TIME ZONE NOT NULL,
    grouping UUID REFERENCES groupings(id),
    seq BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS chunk_placement
(
    shard_id UUID REFERENCES shards(id),
    chunk_hash BYTEA REFERENCES chunk_lookup(hash)
);