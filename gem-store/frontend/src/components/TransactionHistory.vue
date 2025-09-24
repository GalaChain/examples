<template>
  <div class="card">
    <h2>📜 Transaction History</h2>
    
    <div v-if="!walletAddress" class="status error">
      Please connect your wallet first.
    </div>
    
    <div v-else>
      <div class="wallet-info">
        <p><strong>Wallet:</strong> {{ walletAddress }}</p>
        <p><strong>Total Gems:</strong> {{ gemBalance }} 💎</p>
      </div>
      
      <div v-if="isLoading" class="text-center">
        <div class="loading"></div>
        <p>Loading transaction history...</p>
      </div>
      
      <div v-else-if="transactions.length === 0" class="status info">
        No transactions found. Purchase some gems to see your history here!
      </div>
      
      <div v-else class="transaction-list">
        <div 
          v-for="transaction in transactions" 
          :key="transaction.id"
          class="transaction-item"
        >
          <div class="transaction-header">
            <span class="transaction-amount">
              +{{ transaction.gemAmount }} 💎
            </span>
            <span class="transaction-date">
              {{ formatDate(transaction.createdAt) }}
            </span>
          </div>
          <div class="transaction-details">
            <p><strong>GALA Burned:</strong> {{ transaction.galaAmount }} GALA</p>
            <p><strong>Status:</strong> 
              <span :class="getStatusClass(transaction.status)">
                {{ transaction.status }}
              </span>
            </p>
            <p class="transaction-id">
              <strong>Transaction ID:</strong> {{ transaction.transactionId }}
            </p>
          </div>
        </div>
      </div>
      
      <div v-if="error" class="status error mt-3">
        {{ error }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, onMounted } from 'vue'

// Props
const props = defineProps<{
  walletAddress: string
}>()

// State
const transactions = ref([])
const gemBalance = ref(0)
const isLoading = ref(false)
const error = ref('')

// Methods
const fetchTransactionHistory = async () => {
  if (!props.walletAddress) return
  
  try {
    isLoading.value = true
    error.value = ''
    
    const response = await fetch(`/api/transactions/history/${props.walletAddress}`)
    
    if (!response.ok) {
      throw new Error('Failed to fetch transaction history')
    }
    
    const result = await response.json()
    transactions.value = result.transactions || []
    gemBalance.value = result.totalGems || 0
    
  } catch (err) {
    console.error('Error fetching transaction history:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load transaction history'
  } finally {
    isLoading.value = false
  }
}

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return date.toLocaleString()
}

const getStatusClass = (status: string) => {
  switch (status.toLowerCase()) {
    case 'completed':
    case 'success':
      return 'status-success'
    case 'pending':
      return 'status-pending'
    case 'failed':
    case 'error':
      return 'status-error'
    default:
      return 'status-info'
  }
}

// Lifecycle
onMounted(() => {
  if (props.walletAddress) {
    fetchTransactionHistory()
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

.text-center {
  text-align: center;
  padding: 2rem;
}

.transaction-list {
  margin-top: 1rem;
}

.transaction-item {
  border: 1px solid #e1e5e9;
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1rem;
  background: #f8f9fa;
}

.transaction-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.transaction-amount {
  font-size: 1.2rem;
  font-weight: bold;
  color: #28a745;
}

.transaction-date {
  color: #666;
  font-size: 0.9rem;
}

.transaction-details p {
  margin: 0.25rem 0;
  font-size: 0.9rem;
}

.transaction-id {
  font-family: 'Courier New', monospace;
  font-size: 0.8rem;
  color: #666;
  word-break: break-all;
}

.status-success {
  color: #28a745;
  font-weight: bold;
}

.status-pending {
  color: #ffc107;
  font-weight: bold;
}

.status-error {
  color: #dc3545;
  font-weight: bold;
}

.status-info {
  color: #17a2b8;
  font-weight: bold;
}

.mt-3 {
  margin-top: 1rem;
}
</style>
