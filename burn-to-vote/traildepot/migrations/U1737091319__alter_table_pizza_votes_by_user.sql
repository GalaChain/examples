PRAGMA foreign_keys = OFF;

CREATE TABLE __alter_table_pizza_votes_by_user (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7 (id)) DEFAULT (uuid_v7 ()) NOT NULL,
    identity TEXT DEFAULT '',
    pizza_id TEXT DEFAULT '',
    burn_id TEXT DEFAULT ''
) STRICT;

INSERT INTO
    __alter_table_pizza_votes_by_user (id, identity, pizza_id)
SELECT
    id,
    identity,
    pizza_id
FROM
    pizza_votes_by_user;

DROP TABLE pizza_votes_by_user;

ALTER TABLE
    '__alter_table_pizza_votes_by_user' RENAME TO 'pizza_votes_by_user';

PRAGMA foreign_keys = ON;