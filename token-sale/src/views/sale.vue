<template>
  <div class="container">
    <div v-if="loading">
      loading...
    </div>
    <div v-else-if="!sale">
      <h1>Sale not found</h1>
    </div>
    <div v-else>
      <h1>Token Sale</h1>
      <h2>Token Sale ID: {{sale.tokenSaleId}}</h2>
      <h2>Created: {{sale.created}}</h2>
      <h2>Tx ID: {{sale.txId}}</h2>
    </div>
  </div>
</template>
  
<script setup lang="ts">
  import { onMounted, ref } from 'vue';
  import { useRoute } from 'vue-router';
  import { useConnect } from '../composables/connect';
  import { useUserStore } from '../stores/user';
  import { type FetchTokenClassesRequest, type TokenClass } from '@gala-chain/connect';

  const route = useRoute();
  const connect = useConnect();
  const user = useUserStore();
  const loading = ref<boolean>(true);
  const sale = ref<any>();
  const costTokens = ref<TokenClass[]>([]);
  const saleTokens = ref<TokenClass[]>([]);
  
  onMounted(async () => {
    await user.connectWallet();
    if(route.params.tokenSaleId) {
      await getTokenSale(route.params.tokenSaleId as string);
    } else {
      loading.value = false;
    }
  })

  const getTokenSale = async (tokenSaleId: string) => {
    try { 
      loading.value = true;
      sale.value = await connect.fetchTokenSaleById(tokenSaleId);
      const costTokenDto: FetchTokenClassesRequest = {tokenClasses: sale.value.cost.map((c:any) => c.tokenClassKey)};
      costTokens.value = await connect.fetchTokenClasses(costTokenDto);
      const saleTokenDto: FetchTokenClassesRequest = {tokenClasses: sale.value.selling.map((s:any) => s.tokenClassKey)};
      saleTokens.value = await connect.fetchTokenClasses(saleTokenDto);
      const balances = await connect.fetchTokenBalances({owner: user.address}); 
      console.log(balances)
    } catch (error) {
      console.error(error);
    } finally {
      loading.value = false;
    }
  }

  </script>
  
  <style scoped>
  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  
  input {
    max-width: 30rem;
    width:100%;
  }
  
  button {
    margin-top: 2rem;
  }
  </style>