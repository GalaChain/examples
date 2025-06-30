PRAGMA foreign_keys = OFF;

CREATE TABLE __alter_table_pizza_topping_associations (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7 (id)) DEFAULT (uuid_v7 ()) NOT NULL,
    pizza_id BLOB DEFAULT '' REFERENCES pizzas(id) NOT NULL,
    topping_id BLOB DEFAULT '' REFERENCES pizza_toppings(id) NOT NULL,
    quantity INTEGER DEFAULT 1 CHECK(
        quantity IS NULL
        OR quantity BETWEEN 1 AND 3
    )
) STRICT;

INSERT INTO
    __alter_table_pizza_topping_associations (id, pizza_id, topping_id)
SELECT
    id,
    pizza_id,
    topping_id
FROM
    pizza_topping_associations;

DROP TABLE pizza_topping_associations;

ALTER TABLE
    '__alter_table_pizza_topping_associations' RENAME TO 'pizza_topping_associations';

PRAGMA foreign_keys = ON;