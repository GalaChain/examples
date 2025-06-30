CREATE TABLE pizza_votes (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7(id)) DEFAULT (uuid_v7()) NOT NULL,
    pizza_id BLOB DEFAULT '' REFERENCES pizzas(id) UNIQUE NOT NULL,
    total_votes INTEGER DEFAULT 0
) STRICT;