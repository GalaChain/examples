<template>
  <div class="transfer-gala-container">
    <h2>Transfer GALA</h2>
    <div class="transfer-form">
      <div class="input-group">
        <input
          type="text"
          v-model="recipientAddress"
          placeholder="Recipient Address (client|... or eth|...)"
          :disabled="isProcessing"
        />
      </div>
      <div class="input-group">
        <input
          type="number"
          v-model="transferAmount"
          :min="0"
          step="1"
          placeholder="Amount to transfer"
          :disabled="isProcessing"
        />
        <span class="currency">GALA</span>
      </div>
      <small class="fee-notice">Network fee: 1 GALA</small>

      <button 
        @click="transferTokens" 
        :disabled="!isValidTransfer || isProcessing"
      >
        {{ isProcessing ? 'Processing...' : 'Transfer Tokens' }}
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

const recipientAddress = ref('')
const transferAmount = ref<number | null>(null)
const isProcessing = ref(false)
const error = ref('')
const success = ref('')

const isValidTransfer = computed(() => {
  return (
    recipientAddress.value &&
    (recipientAddress.value.startsWith('client|') || recipientAddress.value.startsWith('eth|')) &&
    transferAmount.value !== null &&
    transferAmount.value > 0 &&
    recipientAddress.value.toLowerCase() !== props.walletAddress.toLowerCase()
  )
})

async function transferTokens() {
  if (!isValidTransfer.value || !props.walletAddress) return
  
  error.value = ''
  success.value = ''
  isProcessing.value = true

  try {
    const transferTokensDto = {
      from: props.walletAddress,
      to: recipientAddress.value,
      quantity: transferAmount.value?.toString(),
      tokenInstance: {
        collection: "GALA",
        category: "Unit",
        type: "none",
        additionalKey: "none",
        instance: "0"
      },
      uniqueKey: `january-2025-event-${import.meta.env.VITE_PROJECT_ID}-${Date.now()}`
    }

    const signedTransferDto = await props.metamaskClient.sign("TransferTokens", transferTokensDto)
    
    const response = await fetch(`${import.meta.env.VITE_BURN_GATEWAY_API}/TransferToken`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(signedTransferDto)
    })

    if (!response.ok) {
      throw new Error(`Failed to transfer tokens: ${JSON.stringify(response)}`)
    }

    success.value = `Successfully transferred ${transferAmount.value} GALA to ${recipientAddress.value}`
    transferAmount.value = null
    recipientAddress.value = ''
  } catch (err) {
    console.error(`Error transferring tokens: ${err}`, err)
    error.value = err instanceof Error ? err.message : 'Failed to transfer tokens'
  } finally {
    isProcessing.value = false
  }
}
</script>

<style scoped>
.transfer-gala-container {
  margin: 20px 0;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.transfer-form {
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
  border: 1px solid #ddd;
  border-radius: 4px;
}

input[type="number"] {
  padding-right: 60px;
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