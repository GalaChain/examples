CREATE TABLE pizza_toppings (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7(id)) DEFAULT (uuid_v7()) NOT NULL,
    name TEXT DEFAULT '' NOT NULL UNIQUE,
    description TEXT DEFAULT ''
) STRICT;