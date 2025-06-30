<template>
  <div class="account">
    <div>
      <h2>GalaChain Account/Wallet Details</h2>

      <div>
        <p class="wallet-address">Connected: {{ walletAddress }}</p>
        <Balance :wallet-address="walletAddress" />
        <BurnGala :wallet-address="walletAddress" :metamask-client="metamaskClient" />
        <TransferGala :wallet-address="walletAddress" :metamask-client="metamaskClient" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue"
import Balance from "./Balance.vue"
import BurnGala from "./BurnGala.vue"
import TransferGala from "./TransferGala.vue"
import type { MetamaskConnectClient } from '@gala-chain/connect'

const props = defineProps<{
  walletAddress: string
  metamaskClient: MetamaskConnectClient
}>()

async function connectWallet() {
  try {
    await metamaskClient.connect()
    const address = metamaskClient.getWalletAddress
    walletAddress.value = address.startsWith("0x") ? "eth|" + address.slice(2) : address
    
    // // Check registration
    // try {
    //   await checkRegistration()
    // } catch {
    //   await registerUser()
    // }
    
    isConnected.value = true
  } catch (err) {
    console.error("Error connecting wallet:", err)
  }
}

async function checkRegistration() {
  const response = await fetch(`${import.meta.env.VITE_BURN_GATEWAY_PUBLIC_KEY_API}/GetPublicKey`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ user: walletAddress.value })
  })
  if (!response.ok) throw new Error("User not registered")
}

async function registerUser() {
  const publicKey = await metamaskClient.getPublicKey()
  await fetch(`${import.meta.env.VITE_GALASWAP_API}/CreateHeadlessWallet`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ publicKey: publicKey.publicKey })
  })
}
</script>

<style>
.wallet-address {
  font-family: monospace;
  word-break: break-all;
}
</style>
