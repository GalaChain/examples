<template>
  <div class="wallet-connect">
    <button @click="connectWallet" v-if="!isConnected" class="button">Connect Wallet</button>
    <div v-else>
      <p class="wallet-address">Connected: {{ truncatedAddress }}</p>
      <button @click="registerUser" v-if="!isRegistered" class="button">Register</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { BrowserConnectClient } from '@gala-chain/connect'

const metamaskClient = new BrowserConnectClient();
const isConnected = ref(false)
const walletAddress = ref('')
const isRegistered = ref(false)

const emit = defineEmits(['registrationComplete'])

const truncatedAddress = computed(() => {
  if (walletAddress.value.length > 10) {
    return walletAddress.value
  }
  return walletAddress.value
})

const connectWallet = async () => {
  try {
    await metamaskClient.connect()
    const address = metamaskClient.galaChainAddress;

    walletAddress.value = address;
    isConnected.value = true;
    await checkRegistration()
  } catch (err) {
    console.error('Error connecting wallet:', err)
  }
}

const checkRegistration = async () => {
  try {
    const dto = {
      user: walletAddress.value,
    };

    const url = `${import.meta.env.VITE_GALASWAP_API}/GetPublicKey`;
    const response = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(dto),
    });

    if (!response.ok) {
      console.log(response.status);
      throw new Error(`Failed to check user public key: ${url}`);
    }
    isRegistered.value = !!(await response.json()).Data;
  } catch (err) {
    // not really an error, just means the user is not registered yet
    console.log("User is not registered", err);
    isRegistered.value = false;
  }
}

const registerUser = async () => {
  try {
    const publicKey = await metamaskClient.getPublicKey();

    const registerDto = {
      publicKey: publicKey.publicKey,
    };

    const response = await fetch(
      `${import.meta.env.VITE_GALASWAP_API}/CreateHeadlessWallet`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(registerDto),
      },
    );

    if (!response.ok) {
      console.log(response.status);
      throw new Error(`Failed to register user`);
    }

    isRegistered.value = true;

    emit("registrationComplete");
  } catch (err) {
    console.error("Error registering user:", err);
  }
};

defineExpose({ isConnected, metamaskClient, walletAddress, isRegistered })
</script>

<style scoped>
.wallet-connect {
  margin-bottom: 20px;
}

.wallet-address {
  font-size: 0.9em;
  color: var(--primary-color);
}
</style>