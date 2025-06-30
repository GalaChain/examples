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
  id?: string;
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
  crust: string;
  sauce: string;
  toppings: AssociativeEntity[];
  votes: number;
}

export interface BurnGalaDto {

}

export interface BurnAndSavePizzaSelectionDto {
  pizza: SavePizzaSelectionDto;
  burnDto: BurnGalaDto;
}

export interface PizzaDisplay {
  id: string;
  name: string;
  summary: string;
  contributor: string;
  description: string;
  votes: number;
}

export function pizzaSummary(pizza: PizzaSelectionResDto) {
  const toppingSummary = pizza.toppings
    .sort((a, b) => (a.quantity ?? 0) - (b.quantity ?? 0))
    .map((t) => {
      let multiplier = "";

      if (t.quantity === 2) {
        multiplier = "Double ";
      } else if (t.quantity === 3) {
        multiplier = "Triple ";
      }

      return `${multiplier}${t.name}`;
    })
    .join(", ");

  const crustSummary = pizza.crust ?? "No crust";
  const sauceSummary = pizza.sauce ?? "No sauce";

  return `${toppingSummary} on ${crustSummary} with ${sauceSummary}`;
}