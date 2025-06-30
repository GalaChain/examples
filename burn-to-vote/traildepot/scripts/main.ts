import { 
  addRoute,
  execute,
  HttpError,
  jsonHandler,
  parsePath,
  query,
  StatusCodes
} from "../trailbase.js";

import {
  BURN_GATEWAY_API,
  BURN_GATEWAY_PUBLIC_KEY_API,
  BURN_COST_SUBMIT,
  BURN_COST_VOTE,
  PROJECT_ID
} from "../trailbase.env.js";

import * as sanitize from "./sql-sanitizer/modules/sql_injection.js";

// very basic XSS mitigation
// trailbase has a limited JS runtime, so importing a full featured Node.js module 
// is not possible here. This is sufficent for basic, example, non-production use cases
const xssRe = /[<>/"'`;(){}[\]]/g;

const basicXssSanitize = (input: string) => {
  return input.replace(xssRe, '');
};

// copied from trailbase client npm module 

// https://github.com/trailbaseio/trailbase/blob/0f1d38732fde0de3ae261e21deb2a75549ca9536/trailbase-core/js/client/src/index.ts#L530
/// Decode a base64 string to bytes.
export function base64Decode(base64: string): string {
  return atob(base64);
}

// https://github.com/trailbaseio/trailbase/blob/0f1d38732fde0de3ae261e21deb2a75549ca9536/trailbase-core/js/client/src/index.ts#L535
/// Decode a "url-safe" base64 string to bytes.
export function urlSafeBase64Decode(base64: string): string {
  return base64Decode(base64.replace(/_/g, "/").replace(/-/g, "+"));
}

// end copy/paste from trailbase client

function uint8ArrayToHex(uint8Array: Uint8Array) {
  return Array.from(uint8Array)
    .map(byte => byte.toString(16).padStart(2, '0')) // Convert each byte to a 2-digit hex string
    .join(''); // Join all hex strings together
}

function isValidHexadecimal(str) {
  const hexRegex = /^[0-9A-Fa-f]+$/;
  return hexRegex.test(str);
}

// trailbase uses sqlite uuid7 identifiers stored as binary blobs in sqlite for primary keys 
// in order to reference these binary keys in SQL statements we use a hex representation 
// and the X'' syntax 
function convertIdToHex(id: unknown): string {
  const decodedId = Uint8Array.from(urlSafeBase64Decode(id as string), (c) => c.charCodeAt(0));
  const hexId = uint8ArrayToHex(decodedId);

  if (!isValidHexadecimal(hexId)) {
    throw new HttpError(StatusCodes.INTERNAL_SERVER_ERROR, `Invalid id`)
  }

  return hexId;
}

// data shape shared between basic tables that define an entity id, name, description 
function mapBasicRow(row: unknown[]) {
  const [id, name, description] = row;

  return { id, name, description };
}

// write input from client, only needs id of entity to reference for 
// junction table foreign keys 
export interface AssociativeId {
  id: string;
  quantity?: number | undefined;
}

// full entity, for responses sent to client which need readable display data 
export interface AssociativeEntity extends AssociativeId {
  name: string;
  description?: string;
}

export interface SavePizzaSelectionDto {
  id: string;
  name: string;
  contributor?: string;
  description?: string;
  crust: AssociativeId;
  sauce: AssociativeId;
  toppings: AssociativeId[];
}

export interface PizzaSelectionResDto {
  id: string;
  name: string;
  contributor: string;
  crust: AssociativeEntity;
  sauce: AssociativeEntity;
  toppings: AssociativeEntity[];
}

export interface PizzaResDto {
  id: string;
  name: string;
  contributor: string;
  description: string;
  crust: string;
  crust_id: string;
  sauce: string;
  sauce_id: string;
  toppings: AssociativeEntity[];
  votes?: number;
}

export interface TokenInstanceKey {
  collection: string;
  category: string;
  type: string;
  additionalKey: string;
  instance: string;
}

export interface BurnTokenQuantity {
  tokenInstanceKey: TokenInstanceKey;
  quantity: string;
}

export interface BurnGalaDto {
  tokenInstances: Array<BurnTokenQuantity>;
  owner: string;
  uniqueKey: string;
  signature: string;
}

export interface BurnAndSavePizzaSelectionDto {
  pizza: SavePizzaSelectionDto;
  burnDto: BurnGalaDto;
}

export interface BurnAndVoteDto {
  item_id: string;
  burnDto: BurnAndVoteDto;
}

function verifyBurnDto(burnDto: BurnGalaDto, burnCost: number) {
  if (!Array.isArray(burnDto.tokenInstances) || burnDto.tokenInstances.length !== 1) {
    console.log(`burnDto invalid tokenInstances length`)
    throw new HttpError(StatusCodes.BAD_REQUEST, "Bad Request\n  -- invalid burnDto.tokenInstances length")
  }

  const galaQty: BurnTokenQuantity = burnDto.tokenInstances[0];

  const { collection, category, type, additionalKey } = galaQty.tokenInstanceKey;

  if (collection !== "GALA" || category !== "Unit" || type !== "none" || additionalKey !== "none") {
    console.log(`Recieved burn token instance other than $GALA`)
    throw new HttpError(StatusCodes.BAD_REQUEST, "Bad Request\n -- incorrect burnDto $GALA TokenInstanceKey")
  }

  // dto quantity is serialized BigNumber
  // coerce to number with +
  const burnQuantity: number = +galaQty.quantity;

  if (!(burnQuantity >= burnCost)) {
    console.log(`Insufficnet burn quantity`)
    throw new HttpError(
      StatusCodes.BAD_REQUEST,
      `Bad Request\n  -- insufficient $GALA burn quantity: ` + 
      `${galaQty.quantity}, required: ${burnCost}`
    );
  }

  if (!burnDto.uniqueKey.includes(PROJECT_ID)) {
    console.log(`burnDto signature missing project id: ${PROJECT_ID}, signature: ${burnDto.uniqueKey}`)
    throw new HttpError(
      StatusCodes.BAD_REQUEST,
      `Bad Request\n`
    );
  }

  return burnQuantity;
}

addRoute(
  "GET",
  "/api/pizza-menu",
  jsonHandler(async (req) => {
    const crustRows = await query(
      `SELECT id, name, description FROM pizza_crust_styles`,
      []
    );

    const sauceRows = await query(
      `SELECT * FROM pizza_sauces`,
      []
    );

    const toppingRows = await query(
      `SELECT * FROM pizza_toppings`,
      []
    );

    const crusts = crustRows.map(mapBasicRow);
    const sauces = sauceRows.map(mapBasicRow);
    const toppings = toppingRows.map(mapBasicRow);

    return { crusts, sauces, toppings };
  })
);

addRoute(
  "GET",
  "/api/pizzas",
  jsonHandler(async (req) => {
    const searchParams = parsePath(req.uri).query;

    const bookmark = searchParams.get("bookmark") ?? "";

    // todo: add pagination support
    const pizzas = await query(`
      SELECT pz.id, pz.name, pz.contributor, pz.description, 
      pc.id as crust_id, pc.name as crust_style, 
      ps.id as sauce_id, ps.name as sauce 
      FROM pizzas pz 

      LEFT JOIN pizza_crust_styles pc ON pz.crust_id = pc.id 
      LEFT JOIN pizza_sauces ps ON pz.sauce_id = ps.id;
    `, []);

    const response: PizzaResDto[] = pizzas.map((p: unknown[]) => {
      return {
        id: p[0] as string,
        name: p[1] as string,
        contributor: p[2] as string,
        description: p[3] as string,
        crust_id: p[4] as string,
        crust: p[5] as string,
        sauce_id: p[6] as string,
        sauce: p[7] as string,
        toppings: []
      }
    });

    for (const pizza of response) {
      const pizzaHexId = convertIdToHex(pizza.id);
      const toppings = await query(`
        SELECT pa.quantity, pt.id, pt.name 
        FROM pizza_topping_associations pa 
        LEFT JOIN pizza_toppings pt ON pt.id = pa.topping_id 
        WHERE pa.pizza_id = X'${pizzaHexId}';
        `, []);
  
      pizza.toppings = toppings.map((t) => {
        const topping: AssociativeEntity = {
          quantity: t[0] as number, 
          id: t[1] as string,
          name: t[2] as string
        };
  
        return topping;
      });

      const votesResult = await query(`
        SELECT total_votes FROM pizza_votes 
        WHERE pizza_id = X'${pizzaHexId}';
      `, []);

      if (votesResult.length < 1) {
        pizza.votes = 0;
      } else {
        const votes = votesResult.map((v) => v[0] as number);
        pizza.votes = votes[0]
      }
    }

    return response;
  })
)

addRoute(
  "POST",
  "/api/pizzas",
  jsonHandler(async (req) => {
    let dto: BurnAndSavePizzaSelectionDto;

    if (req.body === undefined) {
      console.log(`Request received with empty body`);
      throw new HttpError(StatusCodes.BAD_REQUEST, "Bad Request\n");
    } else if(typeof req.body === "string") {
      console.log(`Pasring string request body...`)
      try {
        dto = JSON.parse(req.body);
      } catch (e) {
        throw new HttpError(StatusCodes.BAD_REQUEST, "Bad Request - invalid JSON\n");
      }
    } else {
      console.log(`Received parsed json data`)
      dto = req.body as BurnAndSavePizzaSelectionDto;
      console.log(dto);
    }

    const { pizza, burnDto } = dto;

    const burnQuantity = verifyBurnDto(burnDto, BURN_COST_SUBMIT);

    console.log(`begin basic xss sanitize...`);
    const name = basicXssSanitize(pizza.name ?? "");
    const contributor = basicXssSanitize(pizza.contributor ?? "");
    const description = basicXssSanitize(pizza.description ?? "");
    const crust = basicXssSanitize(pizza.crust.id);
    const sauce = basicXssSanitize(pizza.sauce.id);

    console.log(`begin basic sql injection checks...`)
    if (
      sanitize.detectSqlInjection(name) || 
      sanitize.detectSqlInjection(contributor) || 
      sanitize.detectSqlInjection(description) || 
      // fields with base64 encoded UUID7 id foreign keys contain "="
      // setting level to 4 (default 5) slightly loosens the sql injection check 
      sanitize.detectSqlInjection(crust, 4) || 
      sanitize.detectSqlInjection(sauce, 4) || 
      sanitize.detectSqlInjection(pizza.toppings.map((t) => t.id).join(""), 4)
    ) {
      console.log(`SQL injection detected: ${name}, ${contributor}, ${description}, ${crust}, ${sauce}`)
      throw new HttpError(StatusCodes.FORBIDDEN);
    }

    console.log(`converting foreign key ids to hex...`)
    const crustId = convertIdToHex(crust);
    const sauceId = convertIdToHex(sauce);

    console.log(`formatting entity associations...`)
    const toppingIds = pizza.toppings.map((t) => {
      t.id = convertIdToHex(t.id);
      return t;
    });

    console.log(`submit BurnTokens request: ${JSON.stringify(burnDto)}`);

    const burnResponse = await fetch(`${BURN_GATEWAY_API}/BurnTokens`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(burnDto)
    });

    if (!burnResponse.ok) {
      console.log(`Failed to burn $GALA: ${burnResponse.status} -- ${burnResponse.body}`)
      throw new Error(`Failed to burn $GALA`);
    }

    console.log(`🔥🔥 Burned ${burnQuantity} $GALA 🔥🔥`)

    console.log(`saving pizza with foreign keys: ${crust} / ${crustId}, ${sauce} / ${sauceId}`);
    await execute(`
      INSERT INTO 
      pizzas (id, name, contributor, description, crust_id, sauce_id) 
      VALUES (
        uuid_v7(), '${name}', '${contributor}', '${description}', X'${crustId}', X'${sauceId}'
      );
      `, []);

    console.log("saved pizza");

    const savedRow = await query(`SELECT id FROM pizzas WHERE name = '${name}'`, []);

    console.log("retrieved saved pizza");

    const savedId = savedRow[0][0];

    if (typeof savedId !== "string") {
      throw new HttpError(StatusCodes.INTERNAL_SERVER_ERROR, `Error saving: ${savedId}`);
    }

    const pizzaId = convertIdToHex(savedId);

    console.log("pizzaId: ", pizzaId);

    for (const topping of toppingIds) {
      console.log(`insert association: ${pizzaId}, ${topping.id}`);

      const quantity = topping.quantity && isFinite(topping.quantity) ? topping.quantity : 1;

      await execute(`INSERT INTO 
        pizza_topping_associations (id, pizza_id, topping_id, quantity) 
        VALUES (uuid_v7(), X'${pizzaId}', X'${topping.id}', ${quantity});
      `, []);
    }

    pizza.id = savedId;

    return pizza;
  })
);

addRoute(
  "GET",
  "/api/pizzas/:id",
  jsonHandler(async (req) => {
    const pizzaId = req.params.id;

    const pizzaHexId = convertIdToHex(pizzaId);

    const pizzas = await query(`
      SELECT pz.id, pz.name, pz.contributor, pz.description, 
      pc.id as crust_id, pc.name as crust_style, 
      ps.id as sauce_id, ps.name as sauce 
      FROM pizzas pz 
      LEFT JOIN pizza_crust_styles pc ON pz.crust_id = pc.id 
      LEFT JOIN pizza_sauces ps ON pz.sauce_id = ps.id 
      WHERE pz.id = X'${pizzaHexId}';
    `, []);

    if (pizzas.length === 0) {
      throw new HttpError(StatusCodes.NOT_FOUND);
    } else if (pizzas.length > 1) {
      throw new HttpError(StatusCodes.BAD_REQUEST);
    }

    const pizza: PizzaResDto = pizzas.map((p: unknown[]) => {
      return {
        id: p[0] as string,
        name: p[1] as string,
        contributor: p[2] as string,
        description: p[3] as string,
        crust_id: p[4] as string,
        crust: p[5] as string,
        sauce_id: p[6] as string,
        sauce: p[7] as string,
        toppings: []
      }
    })[0];

    const toppings = await query(`
      SELECT pa.quantity, pt.id, pt.name 
      FROM pizza_topping_associations pa 
      LEFT JOIN pizza_toppings pt ON pt.id = pa.topping_id 
      WHERE pa.pizza_id = X'${pizzaHexId}';
      `, []);

    pizza.toppings = toppings.map((t) => {
      const topping: AssociativeEntity = {
        quantity: t[0] as number, 
        id: t[1] as string,
        name: t[2] as string
      };

      return topping;
    });

    return pizza;
  })
);

addRoute(
  "POST",
  "/api/pizzas/:id/vote",
  jsonHandler(async (req) => {
    const pizzaId = req.params.id;

    const pizzaHexId = convertIdToHex(pizzaId);

    let dto: BurnGalaDto;

    if (req.body === undefined) {
      console.log(`Request received with empty body`);
      throw new HttpError(StatusCodes.BAD_REQUEST, "Bad Request\n");
    } else if(typeof req.body === "string") {
      console.log(`Pasring string request body...`)
      try {
        dto = JSON.parse(req.body);
      } catch (e) {
        throw new HttpError(StatusCodes.BAD_REQUEST, "Bad Request - invalid JSON\n");
      }
    } else {
      console.log(`Received parsed json data`)
      dto = req.body as BurnGalaDto;
      console.log(dto);
    }

    const burnQuantity = verifyBurnDto(dto, BURN_COST_VOTE);

    console.log("begin basic xss sanitize...");
    const owner = basicXssSanitize(dto.owner);
    const uniqueKey = basicXssSanitize(dto.uniqueKey);

    console.log("begin basic sql injection check...");
    if (
      sanitize.detectSqlInjection(owner, 4) || 
      sanitize.detectSqlInjection(uniqueKey, 4) || 
      sanitize.detectSqlInjection(pizzaId, 4)
    ) {
      console.log(`SQL Injection possibility flagged: ${owner}, ${uniqueKey}`);
      throw new HttpError(StatusCodes.BAD_REQUEST);
    }
    
    console.log(`submit BurnTokens request: ${JSON.stringify(dto)}`);

    const burnResponse = await fetch(`${BURN_GATEWAY_API}/BurnTokens`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(dto)
    });

    if (!burnResponse.ok) {
      console.log(`Failed to burn $GALA: ${burnResponse.status} -- ${burnResponse.body}`)
      throw new Error(`Failed to burn $GALA`);
    }

    console.log(`🔥🔥 Burned ${burnQuantity} $GALA 🔥🔥`)

    await execute(`INSERT INTO pizza_votes (pizza_id, total_votes)
      VALUES (X'${pizzaHexId}', ${burnQuantity})
      ON CONFLICT(pizza_id) DO UPDATE
      SET total_votes = total_votes + ${burnQuantity};
    `, [])
    .catch((e) => {
      console.log(`Insert to pizza_votes failed ${e}`);
      throw e;
    });

    await execute(`INSERT INTO pizza_votes_by_user (identity, pizza_id, burn_id, votes) 
      VALUES ('${owner}', '${pizzaId}', '${uniqueKey}', ${burnQuantity});
    `, [])
    .catch((e) => {
      console.log(`Insert into pizza_votes_by_user_failed: ${e}`);
      throw e;
    });

    return dto;
  })
);
