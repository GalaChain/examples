<template>
  <div class="list-by-votes">
    <h2>Vote</h2>
    <div v-if="loading">
      <p><i>Fetching current pizza submissions...</i></p>
    </div>
    <div v-else-if="loadError">
      <p><i>Failed to fetch the pizza submissions from the server. Please try again later.</i></p>
    </div>
    <div v-else class="upvote-board">
      <p>Burn $GALA to vote for your favorite submission(s). 
        Every 1 $GALA burned adds 1 vote. Burn more to vote more.
      </p>
      <div v-for="item in voteList" :key="item.id" class="upvote-item">
        <div class="votes">
          <div class="vote-count">        
          <p>{{ item.votes ?? 0 }}</p>
        </div>
          <div class="vote-form">
            <div class="input-group">
              <input 
                type="number" 
                v-model="item.userVoteQty" 
                :min="0"
                step="1"
                placeholder="🔥🔥🔥"
                :disabled="isProcessing"
              />
              <span class="currency">GALA</span>
            </div>
            <small class="fee-notice">Network fee: 1 GALA</small>

            <span class="vote-submit">
            <button 
              @click="submitVote(item)" 
              :disabled="!item.userVoteQty || isProcessing"
            >
              {{ isProcessing ? 'Processing...' : 'Burn & Vote' }}
            </button>
          </span>

            <p v-if="submitError" class="error">{{ error }}</p>
            <p v-if="success" class="success">{{ success }}</p>
          </div>
        </div>
        <div class="item-content">
          <h3 class="item-title">{{ item.name }}</h3>
          <p class="item-description">{{ item.summary }}</p>
          <p class="item-description">{{ item.description }}</p>
          
        </div>
        
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineProps, ref, onMounted } from "vue";
import type { MetamaskConnectClient } from "@gala-chain/connect";

import {
  BurnGalaDto,
  BurnAndVoteDto,
  PizzaDisplay,
  pizzaSummary
} from "../types";

import BigNumber from "bignumber.js";

const apiBase = import.meta.env.VITE_PROJECT_API;
const burnCostVote = ref(new BigNumber(import.meta.env.VITE_BURN_COST_VOTE ?? 1));
const userEnteredBurnQtyForVote = ref<number | null>(null);

const selectedPizza = ref({});

const props = defineProps<{
  walletAddress: string,
  metamaskClient: MetamaskConnectClient
}>();

const voteList = ref([]);

const loading = ref(false);
const isProcessing = ref(false);
const loadError = ref(false);
const submitError = ref("");
const success = ref("");

async function fetchVoteList() {
  loading.value = true;
  loadError.value = false
  
  await fetch(`${apiBase}/api/pizzas`)
    .then((response) => {
      if (!response.ok) throw new Error(`Failed to fetch pizza list for voting`);

      return response.json();
    })
    .then((data) => {
      if (!Array.isArray(data)) {
        throw new Error(`Unexpected response format - ${data}`)
      }

      const displayList = data.map((elem) => {
        const summary = pizzaSummary(elem);

        const { id, name, contributor, description, votes } = elem;
        const userVoteQty = null;
        
        return {
          id, name, contributor, description, summary, votes, userVoteQty
        };
      });

      console.log(`formatted display list`);
      console.log(displayList);

      voteList.value = displayList;
      loading.value = false;
    })
    .catch((e) => {
      loading.value = false;
      loadError.value = true;
    });
}

onMounted(async () => {
  await fetchVoteList();
});

async function confirmBurn(fixedBurnAmount: string) {  
  const burnTokensDto = {
    owner: props.walletAddress,
    tokenInstances: [{
      quantity: fixedBurnAmount,
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

async function submitVote(item) {
  console.log("Submitting vote...");
  console.log(item);

  const userVoteQty = item.userVoteQty;
  const pizzaId = item.id;

  console.log(userVoteQty);
  console.log(pizzaId);

  if (!userVoteQty || userVoteQty < 1 || !props.walletAddress) return;
  
  const dto = await confirmBurn(new BigNumber(userVoteQty).toFixed());

  submitError.value = "";
  success.value = "";
  isProcessing.value = true;

  try {
    const response = await fetch(`${apiBase}/api/pizzas/${pizzaId}/vote`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(dto)
    });

    await fetchVoteList();
  } catch (err) {
    console.error(`Error submitting vote: ${err}`, err);
    submitError.value = err instanceof Error ? err.message : "Failed to submit vote";
  } finally {
    isProcessing.value = false;
  }
}
</script>

<style>
.upvote-board {
  list-style-type: none;
  padding: 0;
  margin: 0;
}

.upvote-item {
  display: flex;
  align-items: center;
  background-color: #111; 
  border: 2.5px solid #33abff; 
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 10px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1); 
  transition: background-color 0.3s ease;
}

.upvote-item:hover {
  background-color: #1f0025;
}

.upvote-item:hover * {
  color: #fff;
}

.upvote-item:hover .vote-form button {
  color: #fff;
}

.item-content {
  flex-grow: 1; 
}

.item-title {
  font-size: 18px;
  font-weight: bold;
  margin: 0;
  color: #ddd;
}

/* Item description styling */
.item-description {
  font-size: 14px;
  color: #dcdcdc;
  margin: 5px 0 10px;
}

/* Vote count styling */
.vote-count {
  font-size: 1.5em;
  text-align: center;
  max-width: 80%;
  font-weight: bold;
  color: #33abff; 
  margin-right: 15px;
  flex-shrink: 0; 
}

.upvote-item:hover .vote-count * {
  color: #33bcff;
}

.input-group {
  max-width: 80%;
  position: relative;
  display: flex;
  align-items: center;
}

.vote-form input[type="number"] {
  width: 100%;
  padding: 8px;
  padding-right: 60px;
  border-radius: 4px;
  font-size: 14px;
  font-weight: bold;
  border: 2.5px solid #33abff;
  background-color: #111;
  color: #007bff;
  border-radius: 4px;
}

.currency {
  position: absolute;
  right: 10px;
  color: #ff33ab;
  font-weight: bold;
}

.upvote-item:hover .currency {
  color: #ff33bc;
}

.vote-form small {
  display: block;
  text-align: center;
  max-width: 80%;
  color: #ff33ab;
  font-weight: bold;
}

.upvote-item:hover .vote-form small {
  color: #ff33cc;
}

.vote-form .vote-submit {
  display: block;
  max-width: 80%;
  text-align: center;

}
.vote-form button {
  padding: 5px 10px;
  font-size: 14px;
  font-weight: bold;
  color: #fff;
  background-color: #dd0088;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.vote-form button:hover {
  background-color: #0056b3;
}

.vote-form button:disabled {
  background-color: #aa0066;
  cursor: not-allowed;
}
</style>