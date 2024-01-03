CREATE TABLE IF NOT EXISTS shards
(
    id UUID PRIMARY KEY,
);

CREATE TABLE IF NOT EXISTS chunk_store
(
    hash BYTEA PRIMARY KEY,
    created DATETIME,
    seq BIGINT,
);

CREATE TABLE IF NOT EXISTS chunk_placement
(
    shard_id UUID REFERENCES shard(id),
    chunk_hash BYTEA REFERNCES chunk_store(hash)   
);