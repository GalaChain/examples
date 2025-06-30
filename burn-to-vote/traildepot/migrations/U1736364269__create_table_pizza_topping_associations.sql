CREATE TABLE pizza_topping_associations (
    id BLOB PRIMARY KEY CHECK(is_uuid_v7(id)) DEFAULT (uuid_v7()) NOT NULL,
    pizza_id BLOB DEFAULT '' REFERENCES pizzas(id) NOT NULL,
    topping_id BLOB DEFAULT '' REFERENCES pizza_toppings(id) NOT NULL
) STRICT;