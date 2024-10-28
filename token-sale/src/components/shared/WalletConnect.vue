<template>
    <button 
    v-if="!userAddress"
    :onclick="userStore.connectWallet"
    class="outline">
        Connect Wallet
    </button>
    <div v-else>
        {{ userAddress.substring(0,10) }}...
    </div>
</template>

<script setup lang="ts">
import { useUserStore } from '@/stores/user';
import { storeToRefs } from 'pinia';

const userStore = useUserStore();
const { address: userAddress } = storeToRefs(userStore)

</script>

<style scoped>
</style>