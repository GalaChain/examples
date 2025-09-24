<template>
  <div class="card">
    <h2>💎 Gem Store</h2>
    
    <div v-if="!walletAddress" class="status error">
      Please connect your wallet first.
    </div>
    
    <div v-else>
      <div class="wallet-info">
        <p><strong>Wallet:</strong> {{ walletAddress }}</p>
        <p><strong>GALA Balance:</strong> {{ galaBalance }} GALA</p>
        <p><strong>Your Gems:</strong> {{ gemBalance }} 💎</p>
      </div>
      
      <div class="gem-packages">
        <div 
          v-for="gemPackage in gemPackages" 
          :key="gemPackage.id"
          class="gem-package"
          :class="{ selected: selectedPackage?.id === gemPackage.id }"
          @click="selectPackage(gemPackage)"
        >
          <h3>{{ gemPackage.gems }} 💎</h3>
          <div class="price">{{ gemPackage.gala }} GALA</div>
          <div class="gems">{{ gemPackage.gems }} gems</div>
        </div>
      </div>
      
      <div v-if="selectedPackage" class="purchase-section">
        <div class="status info">
          <strong>Selected Package:</strong> {{ selectedPackage.gems }} gems for {{ selectedPackage.gala }} GALA
        </div>
        
        <button 
          @click="purchaseGems" 
          :disabled="isProcessing || !canPurchase"
          class="btn btn-danger"
        >
          <span v-if="isProcessing" class="loading"></span>
          {{ isProcessing ? 'Processing...' : `Burn ${selectedPackage.gala} GALA for ${selectedPackage.gems} Gems` }}
        </button>
        
        <div v-if="!canPurchase && selectedPackage" class="status error mt-2">
          Insufficient GALA balance. You need {{ selectedPackage.gala }} GALA but have {{ galaBalance }} GALA.
        </div>
      </div>
      
      <div v-if="status" class="status mt-3" :class="statusType">
        {{ status }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, onMounted } from 'vue'
import { walletService } from '../services/walletService'

// Props
const props = defineProps<{
  walletAddress: string
}>()

// State
const galaBalance = ref(0)
const gemBalance = ref(0)
const selectedPackage = ref(null)
const isProcessing = ref(false)
const status = ref('')
const statusType = ref('')

// Gem packages configuration
const gemPackages = ref([
  { id: 1, gems: 10, gala: 1 },
  { id: 2, gems: 50, gala: 5 },
  { id: 3, gems: 100, gala: 10 },
  { id: 4, gems: 500, gala: 50 }
])

// Computed properties
const canPurchase = computed(() => {
  return selectedPackage.value && galaBalance.value >= selectedPackage.value.gala
})

// Methods
const selectPackage = (gemPackage: any) => {
  selectedPackage.value = gemPackage
  status.value = ''
}

const purchaseGems = async () => {
  if (!selectedPackage.value || !props.walletAddress) return
  
  try {
    isProcessing.value = true
    status.value = ''
    
    // Create burn transaction DTO
    const burnTokensDto = {
      owner: props.walletAddress,
      tokenInstances: [{
        quantity: selectedPackage.value.gala.toString(),
        tokenInstanceKey: {
          collection: "GALA",
          category: "Unit",
          type: "none",
          additionalKey: "none",
          instance: "0"
        }
      }],
      uniqueKey: `gem-purchase-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    }

    // Sign the transaction with MetaMask
    if (!walletService.client) throw new Error('Wallet not connected')
    const signedBurnDto = await walletService.client.sign("BurnTokens", burnTokensDto)
    
    // Submit to backend
    const response = await fetch('/api/transactions/burn', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        signedTransaction: signedBurnDto,
        gemAmount: selectedPackage.value.gems,
        galaAmount: selectedPackage.value.gala,
        walletAddress: props.walletAddress
      })
    })

    if (!response.ok) {
      throw new Error('Failed to process transaction')
    }

    const result = await response.json()
    
    // Update balances
    galaBalance.value -= selectedPackage.value.gala
    gemBalance.value += selectedPackage.value.gems
    
    // Show success message
    status.value = `Successfully purchased ${selectedPackage.value.gems} gems! Transaction: ${result.transactionId}`
    statusType.value = 'success'
    
    // Clear selection
    selectedPackage.value = null
    
  } catch (error) {
    console.error('Error purchasing gems:', error)
    status.value = `Error: ${error instanceof Error ? error.message : 'Failed to purchase gems'}`
    statusType.value = 'error'
  } finally {
    isProcessing.value = false
  }
}

const fetchBalances = async () => {
  try {
    // Fetch GALA balance
    const galaBalanceDto = {
      owner: props.walletAddress,
      collection: "GALA",
      category: "Unit",
      type: "none",
      additionalKey: "none",
      instance: "0"
    }

    const galaResponse = await fetch(
      `${import.meta.env.VITE_BURN_GATEWAY_API}/FetchBalances`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(galaBalanceDto)
      }
    )

    if (galaResponse.ok) {
      const galaResult = await galaResponse.json()
      if (galaResult.Data && galaResult.Data.length > 0) {
        galaBalance.value = parseFloat(galaResult.Data[0].quantity)
      }
    }

    // Fetch gem balance from backend
    const gemResponse = await fetch(`/api/wallet/balance/${props.walletAddress}`)
    if (gemResponse.ok) {
      const gemResult = await gemResponse.json()
      gemBalance.value = gemResult.gemBalance || 0
    }
    
  } catch (error) {
    console.error('Error fetching balances:', error)
  }
}

// Lifecycle
onMounted(() => {
  if (props.walletAddress) {
    fetchBalances()
  }
})
</script>

<style scoped>
.wallet-info {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1.5rem;
}

.wallet-info p {
  margin: 0.25rem 0;
}

.purchase-section {
  margin-top: 1.5rem;
  text-align: center;
}

.mt-2 {
  margin-top: 0.5rem;
}

.mt-3 {
  margin-top: 1rem;
}

.gem-package {
  cursor: pointer;
  transition: all 0.3s ease;
}

.gem-package:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
}

.gem-package.selected {
  border-color: #667eea;
  background-color: #f8f9ff;
}

.gem-package h3 {
  color: #667eea;
  margin-bottom: 0.5rem;
  font-size: 1.5rem;
}

.gem-package .price {
  font-size: 1.1rem;
  font-weight: bold;
  color: #333;
}

.gem-package .gems {
  color: #666;
  margin-top: 0.5rem;
  font-size: 0.9rem;
}
</style>
