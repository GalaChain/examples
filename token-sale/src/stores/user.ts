import { ref, computed, onMounted } from 'vue';
import { defineStore } from 'pinia';
import { useConnect } from '../composables/connect';

export const useUserStore = defineStore('userStore', () => {

    const { connectClient} = useConnect();
    const userAddress = ref<string>();
    const isConnected = computed(() => !!userAddress.value);
    

    const connectWallet = async () => {
        try {
            const connectionResult = await connectClient.connect();
            userAddress.value = connectionResult;
            console.log(`User connected with wallet: ${connectionResult}`)
        } catch (error) {
            console.error(error);
        }
    }

    onMounted(async () => {
        connectClient.on('accountChanged', async (account) => {
            userAddress.value = account as string;
        })
    })

    return { 
        connectWallet,
        address: userAddress,
        isConnected
    }
  })