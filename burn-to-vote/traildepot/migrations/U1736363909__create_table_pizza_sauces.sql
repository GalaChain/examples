CREATE TABLE pizza_sauces (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7(id)) DEFAULT (uuid_v7()) NOT NULL,
    name TEXT DEFAULT '' UNIQUE NOT NULL,
    description TEXT DEFAULT ''
) STRICT;