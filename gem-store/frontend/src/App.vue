<template>
  <div id="app">
    <header class="header">
      <h1>💎 Gem Store</h1>
      <p>GalaChain Wallet Integration Example</p>
    </header>

    <div class="container">
      <nav class="nav">
        <router-link to="/" :class="{ active: $route.path === '/' }">
          Connect Wallet
        </router-link>
        <router-link 
          to="/store" 
          :class="{ active: $route.path === '/store' }"
          v-if="isWalletConnected"
        >
          Gem Store
        </router-link>
        <router-link 
          to="/history" 
          :class="{ active: $route.path === '/history' }"
          v-if="isWalletConnected"
        >
          Transaction History
        </router-link>
      </nav>

      <router-view 
        :wallet-address="walletAddress"
        :is-connected="isWalletConnected"
        @wallet-connected="handleWalletConnected"
        @wallet-disconnected="handleWalletDisconnected"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, provide } from 'vue'
import { walletService } from './services/walletService'

// Wallet state
const isWalletConnected = ref(false)
const walletAddress = ref('')

// Provide wallet state to child components
provide('walletAddress', walletAddress)
provide('isWalletConnected', isWalletConnected)
provide('walletService', walletService)

// Event handlers
const handleWalletConnected = (address: string) => {
  walletAddress.value = address
  isWalletConnected.value = true
}

const handleWalletDisconnected = () => {
  walletAddress.value = ''
  isWalletConnected.value = false
}
</script>
