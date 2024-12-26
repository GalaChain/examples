<template>
  <div class="container">
    <div v-if="showInfo">
      <InfoPage @back="showInfo = false" />
    </div>
    <div v-else>
      <div class="header">
        <h1>GalaChain Burn dApp</h1>
        <button class="info-button" @click="showInfo = true">Info</button>
      </div>
      
      <div v-if="!isConnected" class="connect-section">
        <button @click="connectWallet">Connect Wallet</button>
      </div>

      <div v-else>
        <p class="wallet-address">Connected: {{ walletAddress }}</p>
        <Balance :wallet-address="walletAddress" />
        
        <div class="tabs">
          <button 
            :class="{ active: activeTab === 'burn' }" 
            @click="activeTab = 'burn'"
          >
            Burn GALA
          </button>
          <button 
            :class="{ active: activeTab === 'transfer' }" 
            @click="activeTab = 'transfer'"
          >
            Transfer GALA
          </button>
          <button 
            :class="{ active: activeTab === 'pixel-board' }" 
            @click="activeTab = 'pixel-board'"
          >
            Pixel Board
          </button>
        </div>

        <div class="tab-content">
          <BurnGala 
            v-if="activeTab === 'burn'"
            :wallet-address="walletAddress" 
            :metamask-client="metamaskClient" 
          />
          <TransferGala 
            v-if="activeTab === 'transfer'"
            :wallet-address="walletAddress" 
            :metamask-client="metamaskClient" 
          />
          <PixelBoard 
            v-if="activeTab === 'pixel-board'"
            :wallet-address="walletAddress" 
            :metamask-client="metamaskClient" 
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { MetamaskConnectClient } from '@gala-chain/connect'
import Balance from './components/Balance.vue'
import BurnGala from './components/BurnGala.vue'
import TransferGala from './components/TransferGala.vue'
import PixelBoard from './components/PixelBoard.vue'
import InfoPage from './components/InfoPage.vue'

const metamaskClient = new MetamaskConnectClient()
const isConnected = ref(false)
const walletAddress = ref('')
const showInfo = ref(false)
const activeTab = ref('burn')

async function connectWallet() {
  try {
    await metamaskClient.connect()
    const address = metamaskClient.getWalletAddress
    walletAddress.value = address.startsWith('0x') ? "eth|" + address.slice(2) : address
    
    // Check registration
    try {
      await checkRegistration()
    } catch {
      await registerUser()
    }
    
    isConnected.value = true
  } catch (err) {
    console.error('Error connecting wallet:', err)
  }
}

async function checkRegistration() {
  const response = await fetch(`${import.meta.env.VITE_BURN_GATEWAY_PUBLIC_KEY_API}/GetPublicKey`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user: walletAddress.value })
  })
  if (!response.ok) throw new Error('User not registered')
}

async function registerUser() {
  const publicKey = await metamaskClient.getPublicKey()
  await fetch(`${import.meta.env.VITE_GALASWAP_API}/CreateHeadlessWallet`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ publicKey: publicKey.publicKey })
  })
}
</script>

<style>
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.connect-section {
  text-align: center;
  margin: 40px 0;
}

.wallet-address {
  font-family: monospace;
  word-break: break-all;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.info-button {
  background-color: transparent;
  border: 1px solid var(--primary-color);
  color: var(--primary-color);
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.info-button:hover {
  background-color: var(--primary-color);
  color: var(--background-color);
}

.tabs {
  display: flex;
  gap: 10px;
  margin: 20px 0;
  justify-content: center;
}

.tabs button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  transition: all 0.3s ease;
}

.tabs button.active {
  background: #4CAF50;
  color: white;
  border-color: #4CAF50;
}

.tab-content {
  margin-top: 20px;
}
</style>
