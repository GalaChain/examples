CREATE TABLE pizzas (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7(id)) DEFAULT (uuid_v7()) NOT NULL,
    name TEXT DEFAULT '' NULL,
    contributor TEXT DEFAULT '',
    description TEXT DEFAULT '',
    crust_id BLOB DEFAULT '' REFERENCES pizza_crust_styles(id) NOT NULL,
    sauce_id BLOB DEFAULT '' REFERENCES pizza_sauces(id) NOT NULL
) STRICT;