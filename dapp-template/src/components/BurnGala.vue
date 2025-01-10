<template>
  <div class="burn-container">
    <h2>Burn GALA</h2>
    
    <div class="burn-form">
      <div class="input-group">
        <input 
          type="number" 
          v-model="burnAmount" 
          :min="0"
          step="1"
          placeholder="Amount to burn"
          :disabled="isProcessing"
        />
        <span class="currency">GALA</span>
      </div>
      <small class="fee-notice">Network fee: 1 GALA</small>

      <button 
        @click="burnTokens" 
        :disabled="!isValidAmount || isProcessing"
      >
        {{ isProcessing ? 'Processing...' : 'Burn Tokens' }}
      </button>

      <p v-if="error" class="error">{{ error }}</p>
      <p v-if="success" class="success">{{ success }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { MetamaskConnectClient } from '@gala-chain/connect'

const props = defineProps<{
  walletAddress: string
  metamaskClient: MetamaskConnectClient
}>()

const burnAmount = ref<number | null>(null)
const isProcessing = ref(false)
const error = ref('')
const success = ref('')

const isValidAmount = computed(() => {
  return burnAmount.value !== null && burnAmount.value > 0
})

async function burnTokens() {
  if (!isValidAmount.value || !props.walletAddress) return
  
  error.value = ''
  success.value = ''
  isProcessing.value = true

  try {
    const burnTokensDto = {
      owner: props.walletAddress,
      tokenInstances: [{
        quantity: burnAmount.value?.toString(),
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
    
    const response = await fetch(`${import.meta.env.VITE_GATEWAY_TOKEN_API}/BurnTokens`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(signedBurnDto)
    })

    if (!response.ok) {
      throw new Error('Failed to burn tokens')
    }

    success.value = `Successfully burned ${burnAmount.value} GALA`
    burnAmount.value = null
  } catch (err) {
    console.error(`Error burning tokens: ${err}`, err)
    error.value = err instanceof Error ? err.message : 'Failed to burn tokens'
  } finally {
    isProcessing.value = false
  }
}
</script>

<style scoped>
.burn-container {
  margin: 20px 0;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.burn-form {
  display: flex;
  flex-direction: column;
  gap: 15px;
  max-width: 300px;
  margin: 0 auto;
}

.input-group {
  position: relative;
  display: flex;
  align-items: center;
}

input {
  width: 100%;
  padding: 8px;
  padding-right: 60px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.currency {
  position: absolute;
  right: 10px;
  color: #666;
}

button {
  padding: 10px;
  background-color: #4CAF50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}

.error {
  color: #ff4444;
  text-align: center;
}

.success {
  color: #4CAF50;
  text-align: center;
}

.fee-notice {
  color: #666;
  font-size: 0.8em;
  text-align: right;
  margin-top: -10px;
}
</style>
