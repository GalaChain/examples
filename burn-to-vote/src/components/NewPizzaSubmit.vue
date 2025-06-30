<template>
  <div class="list-by-name">
    <div v-if="loading">
      <p><i>Fetching pizza menu options...</i></p>
    </div>
    <div v-else-if="error">
      <p><i>Failed to fetch the pizza menu from the server. Please try again later.</i></p>
    </div>
    <div class="pizza-selection-form" v-else>
      <h2>Customize Your Pizza</h2>
      <small class="fee-notice">Submission fee: {{ burnCostSubmit.toFixed()}} GALA</small><br />
      <small class="fee-notice">Network fee: 1 GALA</small>

      <h3>Name your delicious submission</h3>
      <div class="input-group">
        <label for="pizza-name">Name</label>
        <input
          id="pizza-name"
          name="pizza-name"
          type="text"
          size="80"
          v-model="pizzaName"
          placeholder="Name the pizza of your dreams..."
          :disabled="isProcessing"
        />
      </div>

      <h3>Describe your delicious submission</h3>
      <div class="input-group">
        <label for="pizza-description">Description</label>
        <textarea
          id="pizza-description"
          name="pizza-description"
          rows="4"
          cols="80"
          v-model="pizzaDescription"
          placeholder="(optional) Describe your artisan composition..."
          :disabled="isProcessing"
        ></textarea>
      </div>

      <h3>Choose Your Crust</h3>
      <select v-model="selectedCrust">
        <option v-for="crust in pizzaMenu.crusts" :key="crust.id" :value="crust">
          {{ crust.name }} - {{ crust.description }}
        </option>
      </select>

      <h3>Choose Your Sauce</h3>
      <select v-model="selectedSauce">
        <option v-for="sauce in pizzaMenu.sauces" :key="sauce.id" :value="sauce">
          {{ sauce.name }} - {{ sauce.description }}
        </option>
      </select>

      <h3>Choose Your Toppings</h3>
      <div v-for="topping in pizzaMenu.toppings" :key="topping.id" class="topping-selector">
        <div class="topping-controls">
          <button 
            @click="decrementTopping(topping.id)" 
            :disabled="selectedToppings[topping.id] === 0">
            -
          </button>
          <span>{{ selectedToppings[topping.id] }}</span>
          <button 
            @click="incrementTopping(topping.id)" 
            :disabled="selectedToppings[topping.id] === 3">
            +
          </button>
        </div>
        <span>{{ topping.name }} - {{ topping.description }}</span>
      </div>

      <button @click="submitSelection">Submit</button>
    </div>
  </div>
</template>


<script setup lang="ts">
import { defineProps, ref, onMounted } from "vue";
import type { MetamaskConnectClient } from "@gala-chain/connect";

import { 
  AssociativeId, 
  AssociativeEntity, 
  SavePizzaSelectionDto, 
  PizzaSelectionResDto, 
  BurnGalaDto,
  BurnAndSavePizzaSelectionDto
} from "../types";

import BigNumber from "bignumber.js";

const apiBase = import.meta.env.VITE_PROJECT_API;
const burnCostSubmit = ref(new BigNumber(import.meta.env.VITE_BURN_COST_SUBMIT ?? 100));

const pizzaMenu = ref({ crusts: [], sauces: [], toppings: [] });

const pizzaName = ref("");
const pizzaDescription = ref("");

const selectedCrust = ref(null);
const selectedSauce = ref(null);
const selectedToppings = ref({});

const props = defineProps<{
  walletAddress: string,
  metamaskClient: MetamaskConnectClient
}>()

const loading = ref(false);
const error = ref(false);
const isProcessing = ref(false);
const submitError = ref("");
const success = ref("");

async function confirmBurn() {  
  const burnTokensDto = {
    owner: props.walletAddress,
    tokenInstances: [{
      quantity: burnCostSubmit.value.toFixed(),
      tokenInstanceKey: {
        collection: "GALA",
        category: "Unit",
        type: "none",
        additionalKey: "none",
        instance: "0"
      }
    }],
    uniqueKey: `january-2025-event-${import.meta.env.VITE_PROJECT_ID}-${Date.now()}`
  }

  const signedBurnDto = await props.metamaskClient.sign("BurnTokens", burnTokensDto)
    .catch(() => undefined);
  
  return signedBurnDto;
}

async function fetchPizzaMenu() {
  loading.value = true;
  error.value = false;

  await fetch(`${apiBase}/api/pizza-menu`)
    .then((response) => {
      if (!response.ok) throw new Error(`Failed to fetch pizza menu`);

      return response.json()
    })
    .then((data) => {
      pizzaMenu.value = data;
      loading.value = false;
    })
    .catch((e) => {
      loading.value = false;
      error.value = true;
    });
}

onMounted(async () => {
  await fetchPizzaMenu();

  if (!Array.isArray(pizzaMenu.value.toppings)) return;

  pizzaMenu.value.toppings.forEach((topping) => {
    selectedToppings.value[topping.id] = 0;
  })
});

// Increment topping quantity
const incrementTopping = (id) => {
  if (selectedToppings.value[id] < 3) {
    selectedToppings.value[id]++;
  }
};

// Decrement topping quantity
const decrementTopping = (id) => {
  if (selectedToppings.value[id] > 0) {
    selectedToppings.value[id]--;
  }
};

async function submitSelection() {
  const userSelections = pizzaMenu.value.toppings.filter((t) => {
    return selectedToppings.value[t.id] !== undefined && selectedToppings.value[t.id] > 0;
  }).map((t) => {
    return {
      id: t.id,
      quantity: selectedToppings.value[t.id] ?? 1
    };
  })

  const selection: SavePizzaSelectionDto = {
    name: pizzaName.value,
    contributor: props.walletAddress,
    description: pizzaDescription.value,
    crust: selectedCrust.value,
    sauce: selectedSauce.value,
    toppings: userSelections,
  };
  console.log("Pizza Selection: ", selection);

  const burnDto = await confirmBurn();

  const dto: BurnAndSavePizzaSelectionDto = {
    pizza: selection,
    burnDto: burnDto
  }

  console.log(dto);

  submitError.value = "";
  success.value = "";
  isProcessing.value = true;

  try {
    const response = await fetch(`${import.meta.env.VITE_PROJECT_API}/api/pizzas`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(dto)
    });

    if (!response.ok) {
      console.log(response.status);
      throw new Error("Failed to submit selection");
    }

    success.value = "Successfully submitted selection";
  } catch (err) {
    console.error(`Error submitting selection: ${err}`, err);
    submitError.value = err instanceof Error ? err.message : "Failed to submit selection";
  } finally {
    isProcessing.value = false;
  }
}
</script>

<style>
.pizza-selection-form {
  max-width: 50em;
  margin: 0 auto;
  font-family: Arial, sans-serif;
}

.topping-controls {
  display: inline-block;
  margin: 0.469em 0.3125em;
}

.topping-controls button {
  margin: 0 0.3125em;
}

button {
  margin-top: 0.625em;
  padding: 0.625em 1.25em;
  font-size: 1em;
  cursor: pointer;
}

.input-group input {
  display: block;
}

.input-group textarea {
  display: block;
}
</style>