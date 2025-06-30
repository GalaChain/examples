CREATE TABLE pizza_crust_styles (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7(id)) DEFAULT (uuid_v7()) NOT NULL,
    name TEXT DEFAULT '' UNIQUE,
    description TEXT DEFAULT ''
) STRICT;