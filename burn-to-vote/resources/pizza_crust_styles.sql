BEGIN TRANSACTION;
INSERT OR IGNORE INTO pizza_crust_styles VALUES(X'0194476ae8657002a81e20a2e219d4d6','Thin Crust','A light and thin crust');
INSERT OR IGNORE INTO pizza_crust_styles VALUES(X'0194476b5a667fd2a6c9a170cb067ee5','Thick Crust','A fluffy thick crust');
INSERT OR IGNORE INTO pizza_crust_styles VALUES(X'0194476c0b957a439009604dd2400667','Wood Fired Crust','A soft, thin crust cooked quickly at high heat in a wood-fired oven');
INSERT OR IGNORE INTO pizza_crust_styles VALUES(X'0194476d568472c286805ef04c05172d','Chicago Style Deep Dish Crust','A deep, thick crust with high edges and lots of cheese and toppings');
INSERT OR IGNORE INTO pizza_crust_styles VALUES(X'0194476de25377b08a2f8d4748daa6bd','New York Style Crust','A thin, foldable crust with a crisp edge');
INSERT OR IGNORE INTO pizza_crust_styles VALUES(X'0194476f05ce72938bcdf6c2d73adce1','Gluten-Free Crust','Made from gluten-free flour designed to mimic the consistency and properties of wheat flour');
INSERT OR IGNORE INTO pizza_crust_styles VALUES(X'0194476fd6727d60ab8324b1485aebeb','Cauliflower Crust','A low-carb, gluten-free, cauliflower-based crust');
COMMIT;