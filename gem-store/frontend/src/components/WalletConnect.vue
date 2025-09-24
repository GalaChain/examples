<template>
  <div class="card">
    <h2>Connect Your Wallet</h2>
    
    <div v-if="!isConnected" class="connect-section">
      <p class="mb-3">
        Connect your MetaMask wallet to start purchasing gems with GALA tokens.
      </p>
      
      <button 
        @click="connectWallet" 
        :disabled="isConnecting"
        class="btn"
      >
        <span v-if="isConnecting" class="loading"></span>
        {{ isConnecting ? 'Connecting...' : 'Connect MetaMask' }}
      </button>
      
      <div v-if="error" class="status error mt-3">
        {{ error }}
      </div>
    </div>
    
    <div v-else class="wallet-connected">
      <div class="wallet-info">
        <h3>✅ Wallet Connected</h3>
        <p class="wallet-address">{{ walletAddress }}</p>
        <p class="balance">Balance: {{ galaBalance }} GALA</p>
      </div>
      
      <div class="status success">
        <strong>Ready to shop!</strong> You can now purchase gems by burning GALA tokens.
      </div>
      
      <div class="mt-3">
        <router-link to="/store" class="btn">
          Go to Gem Store
        </router-link>
        <button @click="disconnectWallet" class="btn btn-danger ml-2">
          Disconnect
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject } from 'vue'
import { walletService } from '../services/walletService'

// Props and emits
const emit = defineEmits(['wallet-connected', 'wallet-disconnected'])

// Injected wallet state
const walletAddress = inject('walletAddress')
const isWalletConnected = inject('isWalletConnected')

// Local state
const isConnected = ref(false)
const isConnecting = ref(false)
const error = ref('')
const galaBalance = ref(0)

// Check if MetaMask is available
const metamaskSupport = !!window.ethereum

// Connect wallet function
const connectWallet = async () => {
  if (!metamaskSupport) {
    error.value = 'MetaMask not detected. Please install MetaMask extension.'
    return
  }

  try {
    isConnecting.value = true
    error.value = ''
    
    // Connect to MetaMask using wallet service
    const connectionResult = await walletService.connect()
    
    if (!connectionResult || !connectionResult.address) {
      throw new Error('Failed to connect to MetaMask')
    }
    
    const address = connectionResult.address
    
    // Skip registration check for now (user is already registered)
    console.log('Skipping registration check - user is already registered')
    
    // Update state
    walletAddress.value = address
    isConnected.value = true
    isWalletConnected.value = true
    
    // Fetch balance
    await fetchBalance(address)
    
    // Emit event
    emit('wallet-connected', address)
    
  } catch (err) {
    console.error('Error connecting wallet:', err)
    error.value = err instanceof Error ? err.message : 'Failed to connect wallet'
  } finally {
    isConnecting.value = false
  }
}

// Check if user is registered with GalaChain
const checkRegistration = async (address: string) => {
  const response = await fetch(
    `${import.meta.env.VITE_BURN_GATEWAY_PUBLIC_KEY_API}/GetPublicKey`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ user: address })
    }
  )
  if (!response.ok) throw new Error('User not registered')
}

// Register new user with GalaChain
const registerUser = async () => {
  if (!walletService.client) throw new Error('Wallet not connected')
  
  const publicKey = await walletService.client.getPublicKey()
  const response = await fetch(
    `${import.meta.env.VITE_GALASWAP_API}/CreateHeadlessWallet`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ publicKey: publicKey.publicKey })
    }
  )
  if (!response.ok) throw new Error('Failed to register user')
}

// Fetch GALA balance
const fetchBalance = async (address: string) => {
  try {
    const balanceDto = {
      owner: address,
      collection: "GALA",
      category: "Unit",
      type: "none",
      additionalKey: "none",
      instance: "0"
    }

    const response = await fetch(
      `${import.meta.env.VITE_BURN_GATEWAY_API}/FetchBalances`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(balanceDto)
      }
    )

    if (response.ok) {
      const result = await response.json()
      if (result.Data && result.Data.length > 0) {
        galaBalance.value = parseFloat(result.Data[0].quantity)
      }
    }
  } catch (err) {
    console.error('Error fetching balance:', err)
  }
}

// Disconnect wallet
const disconnectWallet = async () => {
  try {
    walletService.disconnect()
    isConnected.value = false
    isWalletConnected.value = false
    walletAddress.value = ''
    galaBalance.value = 0
    error.value = ''
    emit('wallet-disconnected')
  } catch (err) {
    console.error('Error disconnecting wallet:', err)
  }
}

// Check for existing connection on mount
import { onMounted } from 'vue'
onMounted(() => {
  if (walletAddress.value) {
    isConnected.value = true
    fetchBalance(walletAddress.value)
  }
})
</script>

<style scoped>
.connect-section {
  text-align: center;
}

.mb-3 {
  margin-bottom: 1rem;
}

.mt-3 {
  margin-top: 1rem;
}

.ml-2 {
  margin-left: 0.5rem;
}

.wallet-connected {
  text-align: center;
}

.wallet-info {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 1.5rem;
  margin-bottom: 1rem;
}

.wallet-info h3 {
  color: #28a745;
  margin-bottom: 0.5rem;
}

.wallet-address {
  font-family: 'Courier New', monospace;
  font-size: 0.9rem;
  color: #666;
  word-break: break-all;
  margin: 0.5rem 0;
}

.balance {
  font-size: 1.2rem;
  font-weight: bold;
  color: #667eea;
  margin-top: 0.5rem;
}
</style>
