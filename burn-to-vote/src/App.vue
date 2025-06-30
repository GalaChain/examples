<template>
  <div class="container">
    <nav>
      <RouterLink to="/">Vote</RouterLink>
      <RouterLink to="/pizza-submission">Submit a Pizza</RouterLink>
      <RouterLink to="/account">Account/Wallet</RouterLink>
      <RouterLink to="/about">About</RouterLink>
    </nav>
    <div>
      <h1>GalaChain Burn and Vote dApp</h1>
      <p>Burn $GALA to upvote a submission. Burn $GALA to submit your own entry.</p>

      <div v-if="!metamaskSupport">
        <p>This application uses the GalaConnect API via Metamask to sign transactions and interact
          with GalaChain.
        </p>
        <p>Visit this site using a browser with the Metamask web extension installed to use the application.</p>
      </div>
      <div v-else-if="!isConnected" class="connect-section">
        <button @click="connectWallet">Connect Wallet</button>
      </div>
      <div v-else>
        <p class="wallet-address">Connected: {{ walletAddress }}</p>
        <RouterView :wallet-address="walletAddress" :metamask-client="metamaskClient" />
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue"
import { createMemoryHistory, createRouter } from "vue-router";
import { MetamaskConnectClient } from "@gala-chain/connect"
import InfoPage from "./components/InfoPage.vue"
import Account from "./components/Account.vue";
import NewPizzaSubmit from "./components/NewPizzaSubmit.vue";
import PizzaList from "./components/ListByVotes.vue";

const metamaskSupport = ref(true);
let metamaskClient: MetamaskConnectClient;
try {
  metamaskClient = new MetamaskConnectClient()
} catch (e) {
  metamaskSupport.value = false;
}

const isConnected = ref(false)
const walletAddress = ref("")
const showInfo = ref(false)

async function connectWallet() {
  if (!metamaskSupport.value) {
    return;
  }

  try {
    await metamaskClient.connect()
    const address = metamaskClient.getWalletAddress
    walletAddress.value = address.startsWith("0x") ? "eth|" + address.slice(2) : address

    // Check registration
    try {
      await checkRegistration()
    } catch {
      await registerUser()
    }

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
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

nav {
  display: flex;
  justify-content: center;
  background-color: #1e00c7;
  padding: 10px 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

nav a {
  color: #fff;
  text-decoration: none;
  padding: 10px 20px;
  margin: 0 10px;
  border-radius: 4px;
  transition: background-color 0.3s ease, color 0.3s ease;
}

nav a:hover {
  background-color: #dd0088;
  color: #f0f0f0;
}

nav a.active {
  background-color: #007bff;
  color: #fff;
  font-weight: bold;
}

.connect-section {
  text-align: center;
  margin: 40px 0;
}

.wallet-address {
  font-family: monospace;
  word-break: break-all;
}
</style>
