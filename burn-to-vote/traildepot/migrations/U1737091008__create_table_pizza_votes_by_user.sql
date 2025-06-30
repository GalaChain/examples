CREATE TABLE pizza_votes_by_user (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7(id)) DEFAULT (uuid_v7()) NOT NULL,
    identity TEXT DEFAULT '',
    pizza_id TEXT DEFAULT ''
) STRICT;